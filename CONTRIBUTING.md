# Contributing to StreamK8s

We are building a revolutionary bridge between local hardware and Kubernetes. Thank you for your interest in contributing!

## 🚀 Workflow
1. **Fork** the repository and create your branch from `master`.
2. **Design First:** If proposing a major change, update **[DESIGN_DECISIONS.md](DESIGN_DECISIONS.md)** first.
3. **Follow the Bridge:** Ensure your code respects the boundaries defined in **[ARCHITECTURE.md](ARCHITECTURE.md)**.
4. **Monkey-Proof UI:** UI changes should be tested for simplicity and intuitive flow.

## 🛠️ Development Environment
- **Backend:** Rust 2021+ (Requires `cargo`).
- **Frontend:** Node.js 20+ (Requires `npm` or `pnpm`).
- **Framework:** Tauri v2 CLI (`cargo tauri`).

## ⚖️ Standards
- **TDD Preferred:** We value tests. Run `cargo test` and `npm run test` (or `npm run build` for type checking).
- **Formatting:** We use `cargo fmt` and `prettier`. Please format before committing.
- **Commits:** Follow conventional commits (e.g., `feat:`, `fix:`, `docs:`).

## 🏷️ Issues & Pull Requests
- Use the provided templates for Bug Reports and Feature Requests.
- Ensure all PRs pass the CI/CD pipeline before requesting review.
