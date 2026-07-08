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
## 12. Helm CLI Integration vs Native Crate (2026-07-08)
**Context:** We need to provide Helm Chart Explorer and Release Manager capabilities in the IDE.
**Decision:** We will use `std::process::Command` to invoke the local `helm` binary instead of a native Rust `helm` crate. 
**Consequences:** Users must have `helm` installed on their OS. This avoids massive binary bloat and compilation complexities associated with compiling the entire Helm Go codebase via FFI or incomplete Rust rewrites. We will return structured JSON from `helm list -o json`.

## 13. Prometheus Metrics Retrieval (2026-07-08)
**Context:** We need cluster-wide metrics for nodes and pods.
**Decision:** We will integrate standard Kubernetes Metrics Server (`metrics.k8s.io`) first for basic pod/node CPU and RAM usage, falling back to an in-cluster Prometheus service if available, using standard `kube-rs` API queries.
**Consequences:** Avoids requiring users to install Prometheus just to see basic CPU/RAM bars on pods.

## 14. AI Profiling Engine (2026-07-08)
**Context:** The core vision of StreamK8s is AI-driven auto-suspension.
**Decision:** We will introduce an `ai_profiler` module in Rust that aggregates telemetry (hardware load + pod metrics) and interfaces with an external LLM (e.g., via a standard HTTP API) or local heuristic model to determine optimal suspension patterns.
**Consequences:** Introduces a dependency on an API key or local model for the "Smart" features.de` for all communications.

### ADR-002: The Bridge Pattern
- **Status:** Accepted
- **Decision:** Implement a strict Command-Query Separation between the Rust core (Hardware/K8s logic) and the Vue frontend.
- **Context:** To ensure the UI remains responsive even when the cluster or system hardware is under heavy load.
- **Consequences:** Requires explicit typing and serialization via `Serde` for all communications.

---
*Add new decisions above this line using the standard ADR format.*
