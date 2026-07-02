# Threshold-Based Autonomous Telemetry Design

## Overview
To fulfill StreamK8s's vision of becoming a fully autonomous OS-level resource negotiator, we are upgrading the hardware telemetry from a hardcoded app-name matcher to an intelligent, threshold-based metric evaluator. This system will dynamically suspend Kubernetes background workloads when local hardware (GPU/CPU) is under sustained heavy load (e.g., from gaming or rendering), and resume them when the load subsides.

## Architecture & Data Flow

1. **Hardware Collector (`src-tauri/src/hardware/collector.rs`)**:
   - Continuously gathers CPU usage (via `sysinfo`) and GPU usage (via `nvml_wrapper`).
   - Maintains a history/buffer of recent metric polls to evaluate sustained load.
   - If `gpu_usage > gpu_suspend_threshold` OR `cpu_usage > cpu_suspend_threshold` for `sustain_duration_seconds`, it emits a `hardware-threshold-exceeded` event.
   - Once the load falls below the threshold (with a possible buffer/hysteresis to prevent flapping) for the same duration, it emits a `hardware-threshold-recovered` event.

2. **Backend Controller (`src-tauri/src/lib.rs`)**:
   - The Tauri event loop listens for `hardware-threshold-exceeded` and `hardware-threshold-recovered`.
   - When exceeded, it triggers the existing Kubernetes auto-suspend logic.
   - When recovered, it triggers the Kubernetes resume logic.

## Configuration & State (`src-tauri/src/config.rs`)

The `AppConfig` will be extended with a `TelemetryConfig` struct (or inline fields) stored in `settings.json`:
- `gpu_suspend_threshold`: `u32` (percentage, default: `80`)
- `cpu_suspend_threshold`: `u32` (percentage, default: `85`)
- `sustain_duration_seconds`: `u32` (seconds, default: `15`)

## Frontend UI (`src/views/SettingsView.vue`)

A new **Hardware Telemetry** card will be added to the settings page:
- **Real-time Monitors**: Small, live progress bars showing current CPU and GPU usage to help the user gauge their idle/load states.
- **Sliders**: For configuring `gpu_suspend_threshold` and `cpu_suspend_threshold`.
- **Duration Input**: A number input for `sustain_duration_seconds`.
- This UI will persist changes back to the Rust backend, dynamically updating the active evaluator loop.

## Error Handling & Edge Cases
- **No GPU Detected**: If `nvml_wrapper` fails to initialize (e.g., AMD GPU or no discrete GPU), the telemetry will fallback to CPU-only evaluation gracefully.
- **Event Flapping**: The `sustain_duration_seconds` ensures that short, transient spikes do not rapidly toggle K8s workloads, which is an expensive operation.
- **Manual Overrides**: The user's existing manual "Auto-Suspend" toggle will act as a master switch. If disabled, these telemetry events will be ignored by the backend.

## Testing Strategy
- **Rust Unit Tests**: Mock the `SystemMetrics` stream and ensure the evaluator correctly triggers `exceeded` and `recovered` states only after the `sustain_duration` is met.
- **Frontend Verification**: Ensure sliders correctly update the `AppConfig` and real-time bars reflect the current system metrics emitted by the backend.
