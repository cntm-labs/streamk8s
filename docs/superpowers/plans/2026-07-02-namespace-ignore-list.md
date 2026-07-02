# Namespace Ignore List Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent Auto-Suspend from scaling down mission-critical namespaces using a combined hardcoded and user-defined ignore list.

**Architecture:** Extend `TelemetryConfig` with `ignored_namespaces`, enforce the boundary in `suspend_namespace` Rust command, and add a UI for managing the config in `SettingsView.vue`.

**Tech Stack:** Rust (Tauri Backend), Vue 3 + Tailwind (Frontend)

## Global Constraints

- Target frontend files: `src/views/SettingsView.vue`
- Target backend files: `src-tauri/src/config.rs`, `src-tauri/src/k8s/scaling.rs`
- Naming conventions: Use `ignored_namespaces` in `TelemetryConfig`
- Hardcoded protected namespaces: `kube-system`, `kube-public`, `kube-node-lease`, `default`
- Backend exact match check only.

---

### Task 1: Update TelemetryConfig Data Model

**Files:**
- Modify: `src-tauri/src/config.rs`

**Interfaces:**
- Produces: `TelemetryConfig` with `pub ignored_namespaces: Vec<String>`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn test_telemetry_config_ignored_namespaces() {
    let config = TelemetryConfig::default();
    assert_eq!(config.ignored_namespaces, Vec::<String>::new());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test config::tests::test_telemetry_config_ignored_namespaces`
Expected: FAIL due to missing field `ignored_namespaces`

- [ ] **Step 3: Write minimal implementation**

Update `TelemetryConfig` in `src-tauri/src/config.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub gpu_suspend_threshold: u32,
    pub cpu_suspend_threshold: u32,
    pub sustain_duration_seconds: u32,
    #[serde(default)]
    pub ignored_namespaces: Vec<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            gpu_suspend_threshold: 80,
            cpu_suspend_threshold: 80,
            sustain_duration_seconds: 300,
            ignored_namespaces: Vec::new(),
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test config::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/config.rs
git commit -m "feat: add ignored_namespaces to TelemetryConfig"
```

### Task 2: Implement Backend Boundary in `suspend_namespace`

**Files:**
- Modify: `src-tauri/src/k8s/scaling.rs`

**Interfaces:**
- Consumes: `AppConfig::load()` config to access `config.telemetry.ignored_namespaces`
- Produces: `suspend_namespace` command which safely ignores scaling down protected namespaces.

- [ ] **Step 1: Write the failing test**

In `src-tauri/src/k8s/scaling.rs` tests module:
```rust
#[tokio::test]
async fn test_suspend_ignored_namespace() {
    use crate::config::{AppConfig, TelemetryConfig};
    
    // We cannot easily test the actual Tauri command `suspend_namespace` here if it requires a running k8s cluster or Tauri AppHandle in its signature.
    // However, since we are adding a check at the top of `suspend_namespace` that returns early, we will just add a check function or test the logic conceptually.
    // Instead of a full E2E, let's write a unit test for a new helper function `is_namespace_ignored`.

    let config = AppConfig {
        telemetry: TelemetryConfig {
            ignored_namespaces: vec!["my-custom-ignore".to_string()],
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(is_namespace_ignored("kube-system", &config));
    assert!(is_namespace_ignored("default", &config));
    assert!(is_namespace_ignored("my-custom-ignore", &config));
    assert!(!is_namespace_ignored("my-app", &config));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test scaling::tests::test_suspend_ignored_namespace`
Expected: FAIL due to `is_namespace_ignored` not defined.

- [ ] **Step 3: Write minimal implementation**

In `src-tauri/src/k8s/scaling.rs`:
```rust
pub fn is_namespace_ignored(namespace: &str, config: &crate::config::AppConfig) -> bool {
    let system_namespaces = ["kube-system", "kube-public", "kube-node-lease", "default"];
    if system_namespaces.contains(&namespace) {
        return true;
    }
    config.telemetry.ignored_namespaces.contains(&namespace.to_string())
}
```

Then update `suspend_namespace`:
```rust
#[tauri::command]
pub async fn suspend_namespace(
    app_handle: tauri::AppHandle,
    context_name: String,
    namespace: String,
) -> Result<(), String> {
    let config = crate::config::AppConfig::load();
    if is_namespace_ignored(&namespace, &config) {
        println!("Ignoring suspend request for protected namespace: {}", namespace);
        return Ok(());
    }

    // existing logic...
```
Make sure `AppConfig` is imported or accessible.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test scaling::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/k8s/scaling.rs
git commit -m "feat: enforce namespace ignore list in suspend_namespace"
```

### Task 3: Frontend UI Integration

**Files:**
- Modify: `src/views/SettingsView.vue`

**Interfaces:**
- Consumes: The `config.telemetry.ignored_namespaces` array.

- [ ] **Step 1: Write minimal implementation**

Since the task brief explicitly bypasses frontend TDD, we implement directly.
In `src/views/SettingsView.vue`:
Add a new ref for the text input:
```typescript
const newIgnoredNamespace = ref('');
const addIgnoredNamespace = () => {
  const ns = newIgnoredNamespace.value.trim();
  if (ns && !config.value.telemetry.ignored_namespaces.includes(ns)) {
    config.value.telemetry.ignored_namespaces.push(ns);
    saveConfig();
  }
  newIgnoredNamespace.value = '';
};

const removeIgnoredNamespace = (ns: string) => {
  config.value.telemetry.ignored_namespaces = config.value.telemetry.ignored_namespaces.filter((n: string) => n !== ns);
  saveConfig();
};
```
And add the UI section below the Telemetry sliders:
```html
<div class="mt-6 border-t border-slate-700/50 pt-6">
  <h3 class="text-sm font-medium text-slate-300 mb-4">Namespace Ignore List</h3>
  <p class="text-xs text-slate-400 mb-3">These namespaces will never be auto-suspended. (System namespaces like kube-system are protected by default).</p>
  
  <div class="flex gap-2 mb-4">
    <input 
      v-model="newIgnoredNamespace" 
      @keyup.enter="addIgnoredNamespace"
      type="text" 
      placeholder="Namespace name..." 
      class="flex-1 bg-slate-900/50 border border-slate-700/50 rounded px-3 py-1.5 text-sm text-slate-300 focus:outline-none focus:border-emerald-500/50"
    />
    <button 
      @click="addIgnoredNamespace"
      class="px-3 py-1.5 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded text-sm hover:bg-emerald-500/20 transition-colors"
    >
      Add
    </button>
  </div>

  <div class="flex flex-wrap gap-2">
    <div 
      v-for="ns in config.telemetry.ignored_namespaces" 
      :key="ns"
      class="inline-flex items-center gap-1.5 px-2 py-1 bg-slate-800 rounded text-xs text-slate-300 border border-slate-700"
    >
      {{ ns }}
      <button @click="removeIgnoredNamespace(ns)" class="text-slate-500 hover:text-red-400">&times;</button>
    </div>
  </div>
</div>
```

- [ ] **Step 2: Run test to verify it passes**

Run: `npm run build`
Expected: Build succeeds without TypeScript errors.

- [ ] **Step 3: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "feat: add namespace ignore list UI to settings"
```
