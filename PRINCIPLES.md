# Engineering Principles

These principles guide the development and maintenance of **StreamK8s**.

## 🛠️ Core Architecture
- **KISS (Keep It Simple, Stupid):** Prioritize clear, maintainable code over complex abstractions.
- **Hardware-First Reactivity:** Every UI update in Vue 3.5 should reflect real-time hardware or K8s state with minimal latency.
- **Memory Safety:** Leverage Rust's ownership model to ensure zero-segfault hardware interfacing.

## ⚖️ Quality Standards
1. **Uncompromising Safety:** Every line of Rust code must prioritize memory safety and secure cluster communication.
2. **Deterministic Orchestration:** Resource suspension logic must be predictable and documented.
3. **Continuous Validation:** No feature is complete without automated tests (Rust unit tests + Playwright/Cypress for UI).

## 🤝 Collaborative Values
- **The "Monkey-Proof" Test:** If a non-expert can't understand the dashboard's intent, the UI needs refactoring.
- **Explicit over Implicit:** All bridge communications between Rust and TS must be explicitly typed using `Serde` or equivalent.
