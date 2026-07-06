# Implementation Plan: Phase 4 Advanced K8s Features

## Feature 1: Full CRD & Resource Viewer (Dynamic API)

### Motivation
To reach OpenLens parity, StreamK8s must dynamically list and edit any resource in the cluster, not just hardcoded structs like `Pod` or `Deployment`.

### Architecture
- **Backend (Rust):**
  - Create a new module `src-tauri/src/k8s/dynamic.rs`.
  - Use `kube::discovery::Discovery` to discover all available API groups and resources in the cluster.
  - Implement a Tauri command `get_api_resources(context_name: &str)` that returns a nested JSON structure of API Groups -> Resources (e.g., `apps/v1 -> Deployment`).
  - Implement a Tauri command `list_dynamic_resource(context_name: &str, group: &str, version: &str, kind: &str, namespace: Option<&str>)` using `kube::api::DynamicObject` and `Api::all_with` or `Api::namespaced_with`.
- **Frontend (Vue):**
  - Add a "Custom Resources" or "All Resources" section in the `Sidebar.vue` (dynamically populated from `get_api_resources`).
  - Create a generic `DynamicResourceTable.vue` that infers columns from the JSON structure (e.g., extracting `.metadata.name`, `.metadata.creationTimestamp`, and arbitrary `.spec` keys if possible).
  - Reuse the existing `YamlEditorModal` for editing these dynamic resources.

## Feature 2: Port Forwarding UI & Manager

### Motivation
Developers need an easy way to tunnel into database pods or internal services directly from the IDE.

### Architecture
- **Backend (Rust):**
  - Create a new module `src-tauri/src/k8s/portforward.rs`.
  - Leverage `kube-rs`'s `Portforwarder` trait. 
  - Manage a thread-safe state (e.g., `Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>`) to keep track of active port-forward sessions by an ID.
  - Tauri commands:
    - `start_port_forward(context, namespace, pod, local_port, remote_port) -> Result<String, String>` (Returns a session ID).
    - `stop_port_forward(session_id) -> Result<(), String>`.
    - `list_active_forwards() -> Vec<ForwardSession>`.
- **Frontend (Vue):**
  - Add a "Network" or "Port Forwards" tab in the `Sidebar.vue` / Activity Bar.
  - In `PodList.vue` or `ResourceTable.vue`, add a "Port Forward" action button to the context menu of a Pod.
  - Create a `PortForwardModal.vue` where users can type the local port and target port.
  - Create a `PortForwardManager.vue` view that lists all active port forwards with a "Stop" button and a clickable localhost link.
