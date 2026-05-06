# Spec: GitHub Repository Initialization for StreamK8s

**Date:** 2026-05-06
**Topic:** Initializing the public GitHub repository for the StreamK8s project under the `cntm-labs` organization.

## 1. Purpose
To establish a public source of truth for the StreamK8s project on GitHub, enabling collaboration and visibility.

## 2. Success Criteria
- A public repository named `streamk8s` exists under the `cntm-labs` organization.
- The repository description accurately reflects the project's mission.
- All local code is pushed to the `master` branch (or `main` if preferred).
- Pre-commit requirements from `GEMINI.md` are fulfilled.

## 3. Proposed Design
### 3.1 Repository Metadata
- **Owner:** `cntm-labs`
- **Name:** `streamk8s`
- **Visibility:** Public
- **Description:** "StreamK8s: The OS-level orchestrator for desktop apps and Kubernetes. Dynamically allocates GPUs and auto-suspends K8s pods for peak performance. Featuring a visual dashboard, AI smart profiling, and an extensible marketplace."

### 3.2 Pre-Push Actions (per GEMINI.md)
- Generate project structure: `tree -a -I 'node_modules|.git|target' > STRUCTURE.tree`
- Format Rust code: `cargo fmt` (if applicable in `src-tauri`)
- Run `pre-commit run --all-files` (if installed)

### 3.3 Execution Steps
1. Create the repository using GitHub CLI (`gh repo create`).
2. Finalize local state (Pre-commit actions).
3. Add/Commit any last-minute changes (like the generated `STRUCTURE.tree`).
4. Push to remote `origin`.

## 4. Risks & Mitigations
- **Permission Issue:** If `MrBT-nano` doesn't have permissions for `cntm-labs`, the creation will fail. *Mitigation:* Error will be reported; user may need to grant access or change owner.
- **Naming Conflict:** If `streamk8s` already exists in `cntm-labs` but isn't visible to the current token. *Mitigation:* `gh` will report the conflict.
