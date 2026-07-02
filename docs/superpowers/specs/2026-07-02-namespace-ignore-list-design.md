# Namespace Ignore List Design Specification

## Overview
As part of the Phase 3 Production-Ready roadmap, StreamK8s needs a robust boundary to prevent the Auto-Suspend feature from scaling down mission-critical or system namespaces. This feature introduces a "Namespace Ignore List" that combines hardcoded system protections with a user-configurable whitelist.

## Architecture & Components

### 1. Data Model (`src-tauri/src/config.rs`)
The `TelemetryConfig` struct will be extended to include:
```rust
pub ignored_namespaces: Vec<String>
```
This configuration will be saved persistently alongside other AppConfig properties.

### 2. Backend Enforcement (`src-tauri/src/k8s/scaling.rs`)
The backend is the ultimate source of truth for boundaries. The `suspend_namespace` command will be updated to act as a secure gatekeeper.

- **Protected System Namespaces**: A hardcoded list of namespaces that can NEVER be suspended:
  - `kube-system`
  - `kube-public`
  - `kube-node-lease`
  - `default`
- **Evaluation**: Before attempting to scale down deployments/statefulsets, `suspend_namespace` will check if the target namespace exists in either the Protected System Namespaces list OR the user's `config.telemetry.ignored_namespaces`.
- **Behavior**: If the namespace is ignored, the command will safely return an `Ok` (or an informative message) without modifying any Kubernetes resources, effectively ignoring the suspend request.

### 3. Frontend UI (`src/views/SettingsView.vue`)
The Telemetry Settings section will be enhanced to allow users to manage their custom ignore list.
- **Input Mechanism**: A text input field where users can type a namespace name and press Enter to add it to a list (chip-based UI).
- **Display**: A list of chips/tags showing currently ignored namespaces. Each chip will have a "remove" button.
- **State Management**: The component will bind to the `ignored_namespaces` array in the `config` object and call the existing `saveConfig` function upon modification.

## Error Handling
- Invalid namespace names (e.g., containing spaces) should be validated in the UI before being added.
- The backend will perform an exact string match for the ignore list check.

## Testing Strategy
- **Rust Unit Tests**: Add tests in `scaling.rs` to verify that `suspend_namespace` immediately returns without action when called with an ignored namespace.
- **Vue UI**: No complex testing required for the UI beyond ensuring the config array updates correctly on save.
