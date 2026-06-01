# Online Registry - Task 1: Backend Discovery & Caching Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a Rust service in the Tauri backend to fetch the remote `registry.json` and cache it locally for offline use.

**Architecture:** Define `RemotePlugin` and `RegistryContent` structs. Implement a `get_remote_registry` Tauri command that uses `reqwest` for fetching, with a local file fallback in `~/.config/streamk8s/registry_cache.json`.

**Tech Stack:** Rust, Tauri, reqwest (with json and rustls-tls features), serde.

---

### Task 1: Update Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add reqwest dependency**

Update `[dependencies]` section in `src-tauri/Cargo.toml`:
```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
```

- [ ] **Step 2: Verify dependency resolution**

Run: `cargo check` in `src-tauri` directory.
Expected: Dependencies resolved and downloaded without errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore: add reqwest dependency for remote registry fetch"
```

### Task 2: Implement Registry Structures

**Files:**
- Modify: `src-tauri/src/plugins/manager.rs`

- [ ] **Step 1: Define RemotePlugin and RegistryContent structs**

Add these structs to `src-tauri/src/plugins/manager.rs`:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemotePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub url: String,
    pub category: String,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryContent {
    pub version: String,
    pub plugins: Vec<RemotePlugin>,
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check` in `src-tauri`.
Expected: Successful compilation.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/plugins/manager.rs
git commit -m "feat: define registry data structures"
```

### Task 3: Implement Registry Fetching and Caching Logic

**Files:**
- Modify: `src-tauri/src/plugins/manager.rs`

- [ ] **Step 1: Implement get_remote_registry command**

Add the command to `src-tauri/src/plugins/manager.rs`:
```rust
#[tauri::command]
pub async fn get_remote_registry() -> Result<Vec<RemotePlugin>, String> {
    let registry_url = "https://raw.githubusercontent.com/cntm-labs/streamk8s/master/registry.json";
    let cache_dir = get_plugin_dir().parent().unwrap().to_path_buf();
    let cache_path = cache_dir.join("registry_cache.json");

    // 1. Attempt to fetch from remote
    let client = reqwest::Client::new();
    let response = client.get(registry_url).send().await;

    match response {
        Ok(res) if res.status().is_success() => {
            let content = res.json::<RegistryContent>().await.map_err(|e| e.to_string())?;
            // Save to cache
            let json = serde_json::to_string(&content).map_err(|e| e.to_string())?;
            fs::write(&cache_path, json).map_err(|e| e.to_string())?;
            Ok(content.plugins)
        }
        _ => {
            // 2. Fallback to cache if remote fails
            if cache_path.exists() {
                let cache_content = fs::read_to_string(&cache_path).map_err(|e| e.to_string())?;
                let content: RegistryContent = serde_json::from_str(&cache_content).map_err(|e| e.to_string())?;
                Ok(content.plugins)
            } else {
                // If no cache, return empty or error? Let's return a mock for now if both fail
                // to avoid blocking UI development, or just error.
                // The plan says "If the fetch fails, attempt to read from the cache file."
                Err("Failed to fetch registry and no cache available".to_string())
            }
        }
    }
}
```

- [ ] **Step 2: Register the command in main.rs**

- Modify: `src-tauri/src/lib.rs` (Tauri v2 usually has commands registered in lib.rs or main.rs)
Wait, let me check `src-tauri/src/lib.rs` and `main.rs`.

- [ ] **Step 3: Verify with cargo check**

Run: `cargo check` in `src-tauri`.
Expected: Successful compilation.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/plugins/manager.rs src-tauri/src/lib.rs
git commit -m "feat: implement registry discovery and caching logic"
```
