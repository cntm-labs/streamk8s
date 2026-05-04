# StreamK8s Design Specification

**Date:** 2026-05-04
**Topic:** StreamK8s Core Architecture and Feature Set
**Status:** Draft

## 1. Overview
StreamK8s is an AI-driven, OS-level Kubernetes IDE designed to surpass existing tools like OpenLens. It addresses common user complaints (resource bloat, forced logins, missing basic features) while introducing groundbreaking OS-level integration for dynamic GPU management. Built on a modern, high-performance stack (Rust, Tauri, Vue), StreamK8s aims to be the definitive tool for local Kubernetes development and AI/ML workload orchestration.

## 2. Core Decisions & Philosophy
Based on the brainstorming session, the following core decisions have been made:

*   **UI/UX Paradigm:** **Unified View**. The interface will merge the visual richness of a traditional dashboard (like Lens) with the high-density, keyboard-driven efficiency of terminal tools (like K9s).
*   **Resource Management:** **Full AI Orchestration**. StreamK8s will actively monitor host OS activity. When heavy desktop applications (e.g., games) are launched, it will automatically throttle non-critical K8s pods and suspend heavy workloads (like ML training) to free up the GPU, resuming them seamlessly when the host application closes.
*   **Extensibility:** **Custom Plugin System**. Instead of maintaining complex backward compatibility with OpenLens extensions, StreamK8s will implement a secure, WebAssembly/JS-based plugin architecture operating within the Tauri sandbox.
*   **Privacy & Accessibility:** 100% Local-first. No forced logins. Essential features (Logs, Shell, Metrics) are built-in, avoiding the "missing features" frustration of recent OpenLens versions.

## 3. Architecture & Components

### 3.1. The Rust Core (Engine)
The backend is written in Rust to guarantee memory safety and maximum performance.
*   **KubeClient:** A robust wrapper around `kube-rs` responsible for all Kubernetes API interactions (v1.30+ compliant, supporting DRA and In-Place Resizing).
*   **OS Monitor:** A cross-platform service that watches for active, high-resource desktop applications (games, rendering software).
*   **AI Orchestrator Engine:** The logic brain that evaluates GPU VRAM usage, interprets the OS Monitor signals, and executes Pod suspension/throttling strategies via the KubeClient.

### 3.2. The Tauri Bridge (IPC)
Acts as the secure communication layer between the Rust backend and the Vue frontend.
*   **Plugin Sandbox:** Manages the execution context and permissions for WebAssembly/JS extensions, ensuring they cannot compromise the host system.

### 3.3. The Vue UI (Frontend)
The user interface, designed for the "Unified View" paradigm.
*   **Smart Dashboard:** Displays real-time cluster health, focusing heavily on GPU metrics (VRAM, Temperature, Time-slicing status) which are often opaque in other tools.
*   **Power-Terminal Grid:** A high-density data grid optimized for keyboard navigation, filtering, and rapid bulk actions.
*   **Visual YAML Builder:** A specialized editor simplifying complex K8s configurations, particularly the new Dynamic Resource Allocation (DRA) syntax.

## 4. Addressing K8s Developer Pain Points

*   **The NVIDIA Stack Struggle:** StreamK8s will include a "Pre-flight GPU Check" to verify compatibility between host drivers and cluster toolkits, preventing cryptic 'Pending' Pod states.
*   **GPU Underutilization:** The UI will natively support and visualize GPU Time-Slicing and MIG (Multi-Instance GPU) partitions.
*   **Resource Bloat:** By eschewing Electron for Tauri/Rust, StreamK8s will maintain a microscopic memory footprint even when managing dozens of clusters.

## 5. Security & Team Features
*   **Security Center:** Built-in integration with open-source scanners (e.g., Trivy) to visualize image vulnerabilities natively.
*   **Local Sharing:** Secure, encrypted export of cluster configurations (Kubeconfig QR/Links) to facilitate team collaboration without relying on a centralized cloud service.

## 6. Testing Strategy
*   **Unit Tests:** Comprehensive coverage for the Rust core logic (especially the AI Orchestrator and OS Monitor).
*   **Integration Tests:** Verifying the Tauri IPC bridge and KubeClient interactions against mock Kubernetes API servers.
*   **E2E Tests:** Automated UI testing to ensure the "Unified View" remains responsive under high data loads.

## 7. Open Questions / Next Steps
*   Finalize the specific IPC command structure between Vue and Rust.
*   Determine the exact mechanism for suspending K8s Pods (e.g., modifying replicas, using custom controllers, or leveraging specific container runtime features).
