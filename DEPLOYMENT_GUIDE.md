# Deployment & Distribution Guide

## 🚀 Overview
StreamK8s is distributed as a native desktop application. This guide covers how to build and package the application for different operating systems.

## 📋 Prerequisites
- **Rust Toolchain:** For compiling the backend core.
- **Node.js:** For building the Vue frontend.
- **OS-specific dependencies:** (e.g., `libwebkit2gtk-4.0-dev` on Linux).

## 🛠️ Build Process
1. **Install Dependencies:**
   ```bash
   npm install
   ```
2. **Build for Production:**
   ```bash
   npm run tauri build
   ```
   This command compiles the Rust core, bundles the Vue frontend, and creates a native installer (e.g., `.deb`, `.app`, `.msi`).

## ⚙️ Configuration
- **Kubeconfig:** StreamK8s automatically searches for `~/.kube/config`. Ensure the user has the necessary permissions to read this file.
- **Environment Variables:** Any production-specific environment variables should be defined in the `.env` file (which is ignored by Git).

## 📊 Distribution
Artifacts are generated in `src-tauri/target/release/bundle/`. These can be distributed via the GitHub Releases page or a custom CDN.
