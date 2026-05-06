# Design Decisions (ADR)

## 💡 Philosophy
This project uses Architectural Decision Records (ADR) to track significant design choices. Every major change to the "Bridge" between OS and K8s must be logged here.

## 📝 Decision Log

### ADR-001: Initial Scaffolding
- **Status:** Accepted
- **Decision:** Use **Rust (Tauri v2)** and **Vue 3.5** to balance high-performance system access with a reactive, beautiful UI.
- **Context:** Bootstrapped to solve the GPU resource hogging problem in local K8s environments.

### ADR-002: The Bridge Pattern
- **Status:** Accepted
- **Decision:** Implement a strict Command-Query Separation between the Rust core (Hardware/K8s logic) and the Vue frontend.
- **Context:** To ensure the UI remains responsive even when the cluster or system hardware is under heavy load.
- **Consequences:** Requires explicit typing and serialization via `Serde` for all communications.

---
*Add new decisions above this line using the standard ADR format.*
