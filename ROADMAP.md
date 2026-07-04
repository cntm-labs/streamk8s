# Project Roadmap

## 📅 Timeline
- **Q1 Foundation:** Core Tauri v2 + Vue 3.5 Engine, Basic Pod Dashboard.
- **Q2 Hardware Bridge:** GPU Detection, Real-time telemetry, Manual Pod Suspension.
- **Q3 AI Orchestration:** Smart Profiling, Auto-suspension logic, AI-assisted troubleshooting.
- **Q4 Ecosystem:** Extensible Marketplace, VSCode integration, Multi-cluster support.

## 🏁 Milestones
- **v0.1 Alpha:** Functional Tauri app with pod listing and local Kubeconfig discovery.
- **v0.5 Beta:** Dynamic GPU allocation logic and suspension hooks.
- **v1.0 Production:** AI-driven profiling and public Extension Marketplace.

## 🚀 Future Vision
Transition from a management tool to a fully autonomous OS-level resource negotiator.

### Phase 1: Foundation
- [x] Implement core Rust core and Tauri bridge.
- [x] Develop responsive Vue dashboard for pod management.
- [x] Integrate `kube-rs` for cluster communication.

### Phase 2: Intelligence
- [x] Implement GPU/CPU telemetry module in Rust.
- [x] Build the "Auto-Suspend" logic core.
- [ ] Develop AI profiling engine for workload pattern recognition.

### Phase 3: Production-Ready (Resilience & UX)
- [x] **Security & Boundaries:** Implement Namespace Whitelist/Ignore List for Auto-Suspend to protect mission-critical workloads.
- [x] **Security & Boundaries:** Handle OS-level permissions (Admin/Root) for hardware metrics with graceful degradation.
- [x] **Resilience & State Recovery:** Implement persistent state for Crash & Reboot Recovery (remembering suspended namespaces).
- [x] **Resilience & State Recovery:** Add Exponential Backoff Retry for Kubernetes API network instability.
- [x] **Self-Performance:** Adaptive Polling for hardware metrics to save battery (e.g., 5s loop when load is low instead of 1s).
- [x] **Testing Coverage:** Implement E2E testing framework (Playwright/WebdriverIO) and Frontend unit tests for threshold events.
- [x] **OS-Level UX:** Integrate Native OS Desktop Notifications for background auto-suspend actions to prevent user confusion.
