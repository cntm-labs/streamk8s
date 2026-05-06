# Security Policy

## 🛡️ Commitment
StreamK8s handles sensitive local hardware telemetry and Kubernetes cluster access. We prioritize a **Security-First** approach to protect both the user's system and their cloud infrastructure.

## 🔐 Security Protocols
1. **Isolation:** Strict boundaries are maintained between local OS processes and the Kubernetes API layer.
2. **Local Secrets:** Kubeconfig data and extension secrets are never stored in plain text or transmitted outside the local system.
3. **Rust Safety:** We utilize Rust's memory safety guarantees to prevent common vulnerabilities like buffer overflows in our hardware bridge.
4. **Extension Sandboxing:** Third-party extensions in our marketplace are subjected to strict permission checks before accessing system resources.

## 📢 Reporting a Vulnerability
Please do not report security vulnerabilities through public GitHub issues. Instead, send a detailed report to:
**security@cntm-labs.com**

## 🔐 Automated Scanning
Mandatory automated security scans for dependencies and code patterns are integrated into our CI/CD pipeline (`.github/workflows/security.yml`).
