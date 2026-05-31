# StreamK8s Extension Developer Guide

## Introduction
StreamK8s is designed to be the default OS-level orchestrator for desktop and Kubernetes workloads. Extensions allow you to add custom functionality, UI components, and automation to the StreamK8s dashboard.

## Environment Setup
To build extensions for StreamK8s, you need the Rust toolchain and the WASM target.

1. **Install Rust:** [rustup.rs](https://rustup.rs/)
2. **Add WASM target:**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Using the Starter Template
The easiest way to start is by using the provided template:

```bash
cp -r templates/rust-plugin my-plugin
cd my-plugin
```

### SDK Integration
Your `Cargo.toml` should include the `streamk8s-sdk`:

```toml
[dependencies]
streamk8s-sdk = { path = "../sdk/rust" }
```

## ABI Specification
StreamK8s provides a low-level ABI for interacting with the host. The `streamk8s-sdk` provides safe wrappers for these functions.

### `get_k8s_resources_count() -> i32`
Returns the number of resources found in the active cluster context.
**Usage in SDK:** `streamk8s_sdk::get_resource_count()`

### `show_notification(code: i32)`
Triggers a system notification via the host application using a predefined status code.
**Usage in SDK:** `streamk8s_sdk::notify(code: i32)`

## UI Definitions (`extension.toml`)
Each plugin must provide an `extension.toml` manifest in its root directory. This defines the plugin metadata and its UI components.

```toml
[extension]
id = "my-plugin"
name = "My Custom Plugin"
description = "Adds a custom dashboard button"
version = "0.1.0"

[[ui.components]]
id = "refresh-btn"
type = "button"
label = "Refresh Resources"
action = "on_action" # Maps to the WASM function
```

### Supported UI Components
- **button**: Triggers a function in your WASM module.
- **input**: Allows user text entry.
- **label**: Displays read-only text.

## Best Practices
1. **Memory Safety**: WASM is sandboxed, but you should still use safe wrappers provided by the SDK.
2. **Error Handling**: Always handle potential `None` or `Err` values from cluster interactions.
3. **Keep it Small**: Minimize the size of your WASM binary for faster loading.
