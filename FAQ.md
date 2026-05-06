# Frequently Asked Questions (FAQ)

## ❓ General
**Q: What is StreamK8s?**
A: StreamK8s is a high-performance desktop orchestrator that dynamically manages system resources (like GPUs) between your local environment and Kubernetes clusters.

**Q: Who is it for?**
A: Developers, data scientists, and power users who run local Kubernetes clusters (like k3s or Minikube) and want to prioritize performance for local apps without manually stopping pods.

## 🛠️ Technical
**Q: How does the "Auto-Suspend" work?**
A: StreamK8s monitors your system's hardware telemetry. If it detects a heavy application (like a game or renderer) demanding GPU/RAM, it uses the Kubernetes API to gracefully suspend background pods until resources are freed.

**Q: Is my Kubeconfig safe?**
A: Yes. StreamK8s runs as a native application on your machine. All communication with your cluster happens locally, and your Kubeconfig data never leaves your system.

## 🤝 Community
**Q: Can I build extensions?**
A: Yes! StreamK8s features a VSCode-like extension marketplace. Check out our [CONTRIBUTING.md](CONTRIBUTING.md) to learn how to get started.
