# Interactive Pod Terminal and Container Selector Design Spec

- **Date:** 2026-06-21
- **Status:** Approved
- **Author:** Antigravity AI
- **Reference:** Competitor analysis (OpenLens terminal/shell capabilities and the alebcay pod menu extension)

## 1. Abstract
This specification details the design for adding a native, interactive Kubernetes Pod Terminal (Shell) and a dynamic Container Selector to the StreamK8s IDE. The solution uses Tauri's event-based communication to bridge a frontend terminal emulator (`xterm.js`) with an asynchronous Kubernetes exec session managed by the Rust backend (`kube-rs` with the `ws` feature). This avoids local proxy server port exposure and integrates seamlessly with the Tauri security architecture.

## 2. Frontend Design

### 2.1 Container Selector Dropdown
- **Location:** Positioned in the upper utility header of [src/components/InspectorPanel.vue](file:///home/mrbt/Desktop/workspaces/k8s-IDE/repositories/streamk8s/src/components/InspectorPanel.vue).
- **Behavior:**
  - On loading a Pod resource, the system parses the Pod payload (`spec.containers[*].name`) to populate a selection dropdown.
  - Defaults to the first container in the list.
  - When the selection changes:
    - **Logs Tab:** Disconnects the existing log event listener, clears logs, and triggers a new `start_log_stream` invoke for the selected container.
    - **Terminal Tab:** Terminates the active terminal session, clears the `xterm` buffer, and starts a new session for the selected container.
    - **Files Tab:** Updates the active container context for the file reader and writer.

### 2.2 Terminal Tab & xterm.js Integration
- **Location:** Added as a new tab named `'Terminal'` in [src/components/InspectorPanel.vue](file:///home/mrbt/Desktop/workspaces/k8s-IDE/repositories/streamk8s/src/components/InspectorPanel.vue).
- **Behavior:**
  - Instantiates `xterm.js` and mounts it on a dedicated container element.
  - Styled to match the dark glassmorphism theme of StreamK8s.
  - **Inputs:** Keyboard entries captured by `xterm.onData(data)` invoke a Tauri command `send_terminal_input(session_id, data)` to stream keys to the backend.
  - **Outputs:** Listens to Tauri window events matching `terminal-stdout-<session_id>` to pipe incoming text bytes directly to `xterm.write`.
  - **Lifecycle Cleanup:** Listens to component unmount and active tab changes to invoke `close_terminal_session(session_id)` to clean up backend tasks.

---

## 3. Backend Design

### 3.1 Registry and Session Management
A global `TerminalSessionManager` is stored in the Tauri application state to handle active interactive streams.

```rust
pub struct TerminalSession {
    pub stdin_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    pub abort_handle: tokio::task::AbortHandle,
}

pub struct TerminalSessionManager {
    pub sessions: std::sync::Mutex<std::collections::HashMap<String, TerminalSession>>,
}
```

This manager is registered as a state in [src-tauri/src/lib.rs](file:///home/mrbt/Desktop/workspaces/k8s-IDE/repositories/streamk8s-tauri/src/lib.rs) during app setup.

### 3.2 Exec Loop Logic
In [src-tauri/src/k8s/inspector.rs](file:///home/mrbt/Desktop/workspaces/k8s-IDE/repositories/streamk8s/src-tauri/src/k8s/inspector.rs):

#### `start_terminal_session`
1. Resolves the Kubernetes client using context config.
2. Initiates automatic shell detection:
   - Tries executing `/bin/bash` with environment variables `COLUMNS=120` and `LINES=35`.
   - If it errors or exits immediately, falls back to `/bin/sh`.
   - If both fail, writes an error message to the stdout stream for the frontend to display: `"Error: No /bin/bash or /bin/sh found in container. This container might be distroless."`
3. Spawns two asynchronous tasks:
   - **Reader Loop (Stdout/Stderr):** Reads output bytes from the connection, processes them using `String::from_utf8_lossy` to prevent serialization failures, and emits a Tauri window event `terminal-stdout-<session_id>`.
   - **Writer Loop (Stdin):** Receives keystroke input byte packets from a Rust `mpsc` channel (`stdin_tx`) and writes them directly to the pod execution process stdin.

#### `send_terminal_input`
- Fetches the active `TerminalSession` by `session_id`.
- Sends the input string bytes to the session's `stdin_tx` channel.

#### `close_terminal_session`
- Removes the session from the registry.
- Invokes `.abort()` on the reader and writer tasks to cleanly terminate execution loops and drop TCP/WebSocket sockets.

---

## 4. Edge Cases & Error Handling

1. **UTF-8 Sanitization:** Low-level ANSI escapes and raw command outputs from containers can contain non-valid UTF-8 bytes. The backend always uses `String::from_utf8_lossy` before converting to JSON events.
2. **Alpine / Minimal / Distroless Containers:** Graceful fallback cascade from `/bin/bash` -> `/bin/sh` -> user-friendly error output.
3. **App Crash/Reload Cleanup:** The backend registry drops all active streams if the Tauri backend is destroyed or reloaded, preventing runaway orphan subprocesses or dangling WebSocket handles on the cluster.
4. **Static Window Sizing:** Passes static `COLUMNS=120` and `LINES=35` environment variables during exec initialization to optimize character fitting.
