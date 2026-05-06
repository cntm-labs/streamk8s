# Troubleshooting Guide

## 🔍 Common Issues

### Issue: GPU Telemetry Not Showing
- **Check:** Ensure you have the latest drivers installed (NVIDIA/AMD).
- **Fix:** On Linux, ensure the user is part of the `video` or `render` group. On Windows/macOS, ensure StreamK8s has permission to access system telemetry.

### Issue: Pods Failing to Suspend
- **Check:** Verify that your Kubeconfig has the necessary permissions to patch deployments.
- **Fix:** Run `kubectl auth can-i patch deployments` to verify access.

### Issue: Application Fails to Launch
- **Check:** Ensure all OS-level dependencies for Tauri v2 are installed.
- **Fix:** On Linux, you may need `libwebkit2gtk-4.1`. Check the [Deployment Guide](DEPLOYMENT_GUIDE.md) for details.

## 🛠️ Debugging
Run the application in a terminal with the environment variable `RUST_LOG=debug` to see detailed logs of the hardware bridge and cluster communication.
