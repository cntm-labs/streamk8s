# GitHub Repository Initialization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a public GitHub repository named `streamk8s` under the `cntm-labs` organization and push the current codebase.

**Architecture:** Use GitHub CLI (`gh`) for repository creation and standard `git` commands for pushing. Follow pre-commit protocols defined in `GEMINI.md`.

**Tech Stack:** GitHub CLI, Git, Shell commands.

---

### Task 1: Pre-push Preparation

**Files:**
- Modify: `STRUCTURE.tree`

- [ ] **Step 1: Generate the project structure file**
Run: `tree -a -I 'node_modules|.git|target' > STRUCTURE.tree`
Expected: `STRUCTURE.tree` is updated with the latest file hierarchy.

- [ ] **Step 2: Format Rust code (Tauri)**
Run: `cd src-tauri && cargo fmt && cd ..`
Expected: Rust source files are formatted according to project standards.

- [ ] **Step 3: Commit pre-push changes**
Run: `git add STRUCTURE.tree && git commit -m "chore: update STRUCTURE.tree before initial push"`
Expected: Local repository is clean and ready for push.

---

### Task 2: GitHub Repository Creation

- [ ] **Step 1: Create the repository on GitHub**
Run: `gh repo create cntm-labs/streamk8s --public --description "StreamK8s: The OS-level orchestrator for desktop apps and Kubernetes. Dynamically allocates GPUs and auto-suspends K8s pods for peak performance. Featuring a visual dashboard, AI smart profiling, and an extensible marketplace."`
Expected: Repository is created on GitHub under `cntm-labs`. `origin` remote is automatically updated or confirmed.

- [ ] **Step 2: Verify remote configuration**
Run: `git remote -v`
Expected: `origin` points to `https://github.com/cntm-labs/streamk8s.git`.

---

### Task 3: Initial Push

- [ ] **Step 1: Push to master branch**
Run: `git push -u origin master`
Expected: All local commits are uploaded to GitHub.

- [ ] **Step 2: Verify repository content**
Run: `gh repo view cntm-labs/streamk8s`
Expected: Repository is visible and contains the pushed files.
