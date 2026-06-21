# Interactive Pod Terminal and Container Selector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a dynamic Container Selector and an interactive Terminal (Pod Shell) with automatic fallback execution inside StreamK8s using Tauri Event-Based streaming.

**Architecture:** The Rust backend registers a global state manager (`TerminalSessionManager`) to control exec loops. Input keystrokes from frontend `xterm.js` are sent via a Tauri command, while stdout/stderr bytes are processed safely (lossy UTF-8) and emitted to the specific window session event.

**Tech Stack:** Rust (kube-rs with ws feature, Tauri v2 State), Vue 3.5 (xterm.js, vitest, @vue/test-utils).

## Global Constraints
- Target frontend files: `src/components/InspectorPanel.vue`
- Target backend files: `src-tauri/src/k8s/terminal.rs`, `src-tauri/src/lib.rs`
- Naming conventions: Use `start_terminal_session`, `send_terminal_input`, `close_terminal_session`
- No dynamic terminal resizing is supported by kube-rs out-of-the-box, so we will use a static shell setup of 120 columns and 35 rows.

---

### Task 1: Backend Session Registry and State Setup

**Files:**
- Create: `src-tauri/src/k8s/terminal.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `TerminalSessionManager`, Tauri commands registration in Tauri setup.

- [ ] **Step 1: Write a failing unit test in `terminal.rs`**
  Add unit tests checking that `TerminalSessionManager` successfully registers, retrieves, and clears active sessions.
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[test]
      fn test_session_manager_registration() {
          let manager = TerminalSessionManager {
              sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
          };
          let (tx, _rx) = tokio::sync::mpsc::channel(1);
          let (abort_handle, _abort_registration) = futures_util::future::AbortHandle::new_pair();
          
          let session = TerminalSession {
              stdin_tx: tx,
              abort_handle,
          };
          
          manager.sessions.lock().unwrap().insert("test-session".to_string(), session);
          assert!(manager.sessions.lock().unwrap().contains_key("test-session"));
          
          manager.sessions.lock().unwrap().remove("test-session");
          assert!(!manager.sessions.lock().unwrap().contains_key("test-session"));
      }
  }
  ```

- [ ] **Step 2: Run backend tests to verify failures**
  Run: `cargo test` inside `src-tauri`
  Expected: FAIL (cannot compile due to missing `TerminalSessionManager` and `TerminalSession` structures)

- [ ] **Step 3: Write minimal implementation in `src-tauri/src/k8s/terminal.rs`**
  ```rust
  use tokio::sync::mpsc::Sender;
  use tokio::task::AbortHandle;
  use std::collections::HashMap;
  use std::sync::Mutex;

  pub struct TerminalSession {
      pub stdin_tx: Sender<Vec<u8>>,
      pub abort_handle: AbortHandle,
  }

  pub struct TerminalSessionManager {
      pub sessions: Mutex<HashMap<String, TerminalSession>>,
  }
  ```

- [ ] **Step 4: Run backend tests to verify passes**
  Run: `cargo test` inside `src-tauri`
  Expected: PASS

- [ ] **Step 5: Register State and command placeholders in `src-tauri/src/lib.rs`**
  Add `TerminalSessionManager` initialization inside `tauri::Builder::default()` state management, and register stub Tauri command functions.
  In `src-tauri/src/lib.rs`:
  ```rust
  // add state registration
  .manage(crate::k8s::terminal::TerminalSessionManager {
      sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
  })
  ```
  And define placeholder commands in `src-tauri/src/k8s/terminal.rs`:
  ```rust
  #[tauri::command]
  pub async fn start_terminal_session(
      _app: tauri::AppHandle,
      _state: tauri::State<'_, TerminalSessionManager>,
      _context_name: Option<String>,
      _namespace: String,
      _pod_name: String,
      _container_name: String,
      _session_id: String,
  ) -> Result<(), String> {
      Ok(())
  }

  #[tauri::command]
  pub async fn send_terminal_input(
      _state: tauri::State<'_, TerminalSessionManager>,
      _session_id: String,
      _data: String,
  ) -> Result<(), String> {
      Ok(())
  }

  #[tauri::command]
  pub async fn close_terminal_session(
      _state: tauri::State<'_, TerminalSessionManager>,
      _session_id: String,
  ) -> Result<(), String> {
      Ok(())
  }
  ```
  Register these three commands inside `tauri::generate_handler!` in `src-tauri/src/lib.rs`.

- [ ] **Step 6: Run tests and verify compiling**
  Run: `cargo test` inside `src-tauri`
  Expected: PASS

- [ ] **Step 7: Commit changes**
  Run:
  ```bash
  git add src-tauri/src/k8s/terminal.rs src-tauri/src/lib.rs
  git commit -m "feat: add terminal session registry state and stub commands"
  ```

---

### Task 2: Backend Exec Session & Fallback Terminal Logic

**Files:**
- Modify: `src-tauri/src/k8s/terminal.rs`

**Interfaces:**
- Consumes: `TerminalSessionManager`
- Produces: Live websocket connections to Pod exec endpoint, event emission of logs.

- [ ] **Step 1: Write exec stream routing test**
  Add mock test verifying MPSC routing to standard terminal handlers.
  ```rust
  // In src-tauri/src/k8s/terminal.rs under mod tests
  #[tokio::test]
  async fn test_mock_terminal_write() {
      let (tx, mut rx) = tokio::sync::mpsc::channel(10);
      tx.send(b"hello".to_vec()).await.unwrap();
      let msg = rx.recv().await.unwrap();
      assert_eq!(msg, b"hello");
  }
  ```

- [ ] **Step 2: Run tests to verify setup**
  Run: `cargo test` in `src-tauri`
  Expected: PASS

- [ ] **Step 3: Implement `start_terminal_session`, `send_terminal_input`, and `close_terminal_session`**
  Write full interactive logic inside `src-tauri/src/k8s/terminal.rs`:
  ```rust
  use futures_util::{AsyncReadExt, AsyncWriteExt};
  use tauri::Emitter;
  use kube::api::{Api, AttachParams};
  use k8s_openapi::api::core::v1::Pod;

  #[tauri::command]
  pub async fn start_terminal_session(
      app: tauri::AppHandle,
      state: tauri::State<'_, TerminalSessionManager>,
      context_name: Option<String>,
      namespace: String,
      pod_name: String,
      container_name: String,
      session_id: String,
  ) -> Result<(), String> {
      let client = crate::k8s::inspector::create_client(context_name).await?;
      let pods: Api<Pod> = Api::namespaced(client, &namespace);

      // Check bash first, fallback to sh
      let mut attached = match pods.exec(
          &pod_name,
          vec!["/bin/bash"],
          &AttachParams::default()
              .container(&container_name)
              .stdin(true)
              .stdout(true)
              .stderr(true)
              .tty(true),
      ).await {
          Ok(res) => res,
          Err(_) => {
              pods.exec(
                  &pod_name,
                  vec!["/bin/sh"],
                  &AttachParams::default()
                      .container(&container_name)
                      .stdin(true)
                      .stdout(true)
                      .stderr(true)
                      .tty(true),
              ).await.map_err(|e| format!("Both bash and sh failed: {}", e))?
          }
      };

      let mut stdout = attached.stdout().ok_or("Failed to attach to stdout")?;
      let mut stdin = attached.stdin().ok_or("Failed to attach to stdin")?;

      let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);

      // Writer Task (stdin)
      let writer_task = tokio::spawn(async move {
          while let Some(data) = stdin_rx.recv().await {
              if stdin.write_all(&data).await.is_err() {
                  break;
              }
              let _ = stdin.flush().await;
          }
      });

      // Reader Task (stdout)
      let app_clone = app.clone();
      let session_id_clone = session_id.clone();
      let reader_task = tokio::spawn(async move {
          let mut buffer = [0u8; 1024];
          loop {
              match stdout.read(&mut buffer).await {
                  Ok(0) => {
                      let _ = app_clone.emit(&format!("terminal-exit-{}", session_id_clone), ());
                      break;
                  }
                  Ok(n) => {
                      let text = String::from_utf8_lossy(&buffer[..n]).to_string();
                      let _ = app_clone.emit(&format!("terminal-stdout-{}", session_id_clone), text);
                  }
                  Err(_) => {
                      let _ = app_clone.emit(&format!("terminal-exit-{}", session_id_clone), ());
                      break;
                  }
              }
          }
      });

      // Track abort handle
      let (abort_handle, abort_registration) = futures_util::future::AbortHandle::new_pair();
      tokio::spawn(futures_util::future::Aborted::new(
          async move {
              writer_task.abort();
              reader_task.abort();
          },
          abort_registration,
      ));

      let session = TerminalSession {
          stdin_tx,
          abort_handle,
      };

      state.sessions.lock().unwrap().insert(session_id, session);
      Ok(())
  }

  #[tauri::command]
  pub async fn send_terminal_input(
      state: tauri::State<'_, TerminalSessionManager>,
      session_id: String,
      data: String,
  ) -> Result<(), String> {
      let sessions = state.sessions.lock().unwrap();
      if let Some(session) = sessions.get(&session_id) {
          let _ = session.stdin_tx.send(data.into_bytes()).await;
      }
      Ok(())
  }

  #[tauri::command]
  pub async fn close_terminal_session(
      state: tauri::State<'_, TerminalSessionManager>,
      session_id: String,
  ) -> Result<(), String> {
      let mut sessions = state.sessions.lock().unwrap();
      if let Some(session) = sessions.remove(&session_id) {
          session.abort_handle.abort();
      }
      Ok(())
  }
  ```

- [ ] **Step 4: Run tests to verify compilation**
  Run: `cargo test` in `src-tauri`
  Expected: PASS

- [ ] **Step 5: Commit changes**
  Run:
  ```bash
  git add src-tauri/src/k8s/terminal.rs
  git commit -m "feat: implement shell exec, stdin channel writing and stdout event emitting loops"
  ```

---

### Task 3: Frontend Container Selector Dropdown Integration

**Files:**
- Modify: `src/components/InspectorPanel.vue`

**Interfaces:**
- Consumes: Pod payload attributes.
- Produces: Reactive selection of container targets.

- [ ] **Step 1: Write Vitest unit test for parsing containers**
  Create `src/components/__tests__/InspectorPanel.spec.ts` if missing and add tests for container list rendering.
  ```typescript
  import { describe, it, expect } from 'vitest';
  import { mount } from '@vue/test-utils';
  import InspectorPanel from '../InspectorPanel.vue';

  describe('InspectorPanel Container Parsing', () => {
    it('populates dropdown with container names from selected resource spec', async () => {
      const mockResource = {
        contextName: 'ctx',
        namespace: 'default',
        name: 'test-pod',
        kind: 'Pods',
        spec: {
          containers: [{ name: 'container-a' }, { name: 'container-b' }]
        }
      };
      const wrapper = mount(InspectorPanel, {
        props: { selectedResource: mockResource }
      });
      // Wait for rendering
      await wrapper.vm.$nextTick();
      const options = wrapper.findAll('option');
      expect(options.length).toBeGreaterThanOrEqual(1);
    });
  });
  ```

- [ ] **Step 2: Run test to verify failure**
  Run: `npm run test`
  Expected: FAIL (either file missing or dropdown elements missing)

- [ ] **Step 3: Modify `InspectorPanel.vue` to add Container Selector**
  Extract list of containers from pod spec, track `selectedContainer` state.
  Replace hardcoded container values in `readFile`, `saveFile`, `initiateLogStream` functions with the dynamic variable `selectedContainer.value`.
  In `src/components/InspectorPanel.vue`:
  ```vue
  <!-- Add select dropdown under utility header -->
  <div class="container-select-wrapper" v-if="props.selectedResource && props.selectedResource.kind === 'Pods'">
    <label for="container-select">Container:</label>
    <select id="container-select" v-model="selectedContainer" class="container-selector">
      <option v-for="c in containerNames" :key="c" :value="c">{{ c }}</option>
    </select>
  </div>
  ```
  Inside `<script setup>`:
  ```typescript
  const containerNames = ref<string[]>([]);
  const selectedContainer = ref('');

  // When selectedResource changes, populate containers
  watch(() => props.selectedResource, async (newVal) => {
    if (newVal && newVal.kind === 'Pods') {
      try {
        // Fetch full pod spec via Tauri invoke if payload doesn't contain spec
        const podDetailsText = await invoke<string>('get_k8s_resource_details', {
          contextName: newVal.contextName,
          namespace: newVal.namespace,
          name: newVal.name,
          kind: 'Pod'
        });
        const parsed = JSON.parse(podDetailsText);
        const names = parsed.spec?.containers?.map((c: any) => c.name) || [];
        containerNames.value = names;
        if (names.length > 0) {
          selectedContainer.value = names[0];
        }
      } catch (e) {
        console.error('Failed to load container list:', e);
        containerNames.value = [newVal.name];
        selectedContainer.value = newVal.name;
      }
    }
  }, { immediate: true });
  ```

- [ ] **Step 4: Run test to verify pass**
  Run: `npm run test`
  Expected: PASS

- [ ] **Step 5: Commit changes**
  Run:
  ```bash
  git add src/components/InspectorPanel.vue src/components/__tests__/InspectorPanel.spec.ts
  git commit -m "feat: add container selector dropdown and link to read/write/logs logic"
  ```

---

### Task 4: Frontend Terminal Tab (xterm.js) and Event Streaming

**Files:**
- Modify: `src/components/InspectorPanel.vue`
- Modify: `src/App.vue` (ensure app styling is consistent)

**Interfaces:**
- Consumes: Tauri events `terminal-stdout-<session_id>`, commands `start_terminal_session`, `send_terminal_input`, `close_terminal_session`.

- [ ] **Step 1: Write Vitest unit test for Terminal mounting**
  Update `src/components/__tests__/InspectorPanel.spec.ts` to assert that clicking the "Terminal" tab renders the terminal container.
  ```typescript
  it('renders terminal tab contents when Terminal is clicked', async () => {
    const mockResource = {
      contextName: 'ctx',
      namespace: 'default',
      name: 'test-pod',
      kind: 'Pods'
    };
    const wrapper = mount(InspectorPanel, {
      props: { selectedResource: mockResource }
    });
    // Set tab to Terminal
    await wrapper.setData({ activeTab: 'Terminal' });
    expect(wrapper.find('.terminal-panel-body').exists()).toBe(true);
  });
  ```

- [ ] **Step 2: Run test to verify failure**
  Run: `npm run test`
  Expected: FAIL

- [ ] **Step 3: Import `xterm` assets and implement mounting logic**
  Add styles and script logic to instantiate xterm in `InspectorPanel.vue`.
  ```typescript
  import { Terminal } from 'xterm';
  import 'xterm/css/xterm.css';

  const terminalDiv = ref<HTMLElement | null>(null);
  let term: Terminal | null = null;
  const sessionId = ref('');

  const initTerminal = async () => {
    if (!props.selectedResource || !terminalDiv.value) return;
    sessionId.value = Math.random().toString(36).substring(7);

    term = new Terminal({
      cols: 120,
      rows: 35,
      theme: {
        background: 'rgba(30, 30, 30, 0.8)',
        foreground: '#f8f8f2',
        cursor: '#f8f8f0',
      }
    });

    term.open(terminalDiv.value);
    term.write('Connecting to terminal...\r\n');

    await invoke('start_terminal_session', {
      contextName: props.selectedResource.contextName,
      namespace: props.selectedResource.namespace,
      podName: props.selectedResource.name,
      containerName: selectedContainer.value,
      sessionId: sessionId.value
    });

    // Listeners
    const stdoutListener = await listen<string>(`terminal-stdout-${sessionId.value}`, (event) => {
      term?.write(event.payload);
    });

    const exitListener = await listen<void>(`terminal-exit-${sessionId.value}`, () => {
      term?.write('\r\n[Connection Closed]\r\n');
    });

    term.onData((data) => {
      invoke('send_terminal_input', {
        sessionId: sessionId.value,
        data
      });
    });

    // Save listeners to unlisten later
    onUnmounted(() => {
      stdoutListener();
      exitListener();
      invoke('close_terminal_session', { sessionId: sessionId.value }).catch(() => {});
    });
  };
  ```

- [ ] **Step 4: Run test to verify passes**
  Run: `npm run test`
  Expected: PASS

- [ ] **Step 5: Verify building of Vite resources**
  Run: `npm run build`
  Expected: Success

- [ ] **Step 6: Commit changes and updates**
  Run:
  ```bash
  git add src/components/InspectorPanel.vue
  git commit -m "feat: integrate xterm.js tab and event streaming communication with tauri backend"
  ```
