# Design Decisions (ADR)

## 💡 Philosophy
This project uses Architectural Decision Records (ADR) to track significant design choices. Every major change to the "Bridge" between OS and K8s must be logged here.

## 📝 Decision Log

### ADR-001: Initial Scaffolding
- **Status:** Accepted
- **Decision:** Use **Rust (Tauri v2)** and **Vue 3.5** to balance high-performance system access with a reactive, beautiful UI.
- **Context:** Bootstrapped to solve the GPU resource hogging problem in local K8s environments.

### 10. Dynamic CRD & Resource Viewer Implementation (2026-07-06)
**Context:** To elevate StreamK8s to a fully-featured Kubernetes IDE like OpenLens, it must support all Kubernetes resources, including Custom Resource Definitions (CRDs). Hardcoding structs for every possible CRD is impossible.
**Decision:** We will use the `kube-rs` Dynamic API (`DynamicObject` and `Api::all_with` / `Api::namespaced_with`). We will fetch the `ApiResource` metadata using the `Discovery` client, allowing us to list, view, and edit any Kubernetes resource generically.
**Consequences:** This trades compile-time type safety for complete flexibility. The frontend will need a generic YAML/JSON viewer and table renderer that can adapt to arbitrary resource structures.

## 11. Native Port Forwarding via kube-rs (2026-07-06)
**Context:** Port forwarding is a critical developer workflow. Running external `kubectl port-forward` commands via shell can be fragile and hard to manage across platforms.
**Decision:** We will use `kube-rs` built-in `Portforward` trait and websocket (`ws`) feature. We will manage port forwarding sessions in the Rust backend using `tokio` tasks, tracking active sessions in a Thread-Safe state manager, and streaming logs/status to the frontend via Tauri events.
**Consequences:** Requires upgrading/enabling `ws` feature in `kube-rs` (already enabled). We need to handle port conflict resolution, lifecycle management (stopping forwards), and provide a dedicated UI in the frontend.de` for all communications.

### ADR-002: The Bridge Pattern
- **Status:** Accepted
- **Decision:** Implement a strict Command-Query Separation between the Rust core (Hardware/K8s logic) and the Vue frontend.
- **Context:** To ensure the UI remains responsive even when the cluster or system hardware is under heavy load.
- **Consequences:** Requires explicit typing and serialization via `Serde` for all communications.

---
*Add new decisions above this line using the standard ADR format.*
