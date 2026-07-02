use crate::config::TelemetryConfig;
use nvml_wrapper::Nvml;
use serde::Serialize;
use std::time::Instant;
use sysinfo::System;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub ram_usage: f32,
    pub gpu_usage: Option<f32>,
    pub gpu_mem_usage: Option<f32>,
}

pub fn check_for_heavy_apps(sys: &System, app: &AppHandle) {
    let heavy_apps = ["adobe premiere", "cyberpunk", "chrome", "firefox", "code"];

    for (_pid, process) in sys.processes() {
        let name = process.name().to_lowercase();
        if heavy_apps.iter().any(|&ha| name.contains(ha)) && process.cpu_usage() > 20.0 {
            app.emit("heavy-app-detected", name).ok();
            break;
        }
    }
}

pub fn collect_metrics(sys: &mut System, nvml: &Option<Nvml>) -> SystemMetrics {
    // In sysinfo 0.30, traits like SystemExt and CpuExt are no longer needed.
    // Methods are now implemented directly on the types.
    sys.refresh_all();

    let cpu = sys.global_cpu_info().cpu_usage();
    let ram = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;

    let mut gpu_load = None;
    let mut gpu_mem = None;

    if let Some(n) = nvml {
        if let Ok(device) = n.device_by_index(0) {
            if let Ok(util) = device.utilization_rates() {
                gpu_load = Some(util.gpu as f32);
            }
            if let Ok(mem) = device.memory_info() {
                gpu_mem = Some((mem.used as f32 / mem.total as f32) * 100.0);
            }
        }
    }

    SystemMetrics {
        cpu_usage: cpu,
        ram_usage: ram,
        gpu_usage: gpu_load,
        gpu_mem_usage: gpu_mem,
    }
}

#[derive(Debug)]
pub enum EvaluatorState {
    Normal,
    Suspended,
}

pub struct HardwareEvaluator {
    pub state: EvaluatorState,
    pub first_exceeded: Option<Instant>,
    pub first_recovered: Option<Instant>,
}

impl HardwareEvaluator {
    pub fn new() -> Self {
        Self {
            state: EvaluatorState::Normal,
            first_exceeded: None,
            first_recovered: None,
        }
    }

    pub fn evaluate(
        &mut self,
        metrics: &SystemMetrics,
        config: &TelemetryConfig,
    ) -> Option<&'static str> {
        let is_heavy = metrics.cpu_usage > config.cpu_suspend_threshold as f32
            || metrics.gpu_usage.unwrap_or(0.0) > config.gpu_suspend_threshold as f32;

        match self.state {
            EvaluatorState::Normal => {
                if is_heavy {
                    if let Some(t) = self.first_exceeded {
                        if t.elapsed().as_secs() >= config.sustain_duration_seconds as u64 {
                            self.state = EvaluatorState::Suspended;
                            self.first_exceeded = None;
                            return Some("hardware-threshold-exceeded");
                        }
                    } else {
                        self.first_exceeded = Some(Instant::now());
                    }
                } else {
                    self.first_exceeded = None;
                }
            }
            EvaluatorState::Suspended => {
                if !is_heavy {
                    if let Some(t) = self.first_recovered {
                        if t.elapsed().as_secs() >= config.sustain_duration_seconds as u64 {
                            self.state = EvaluatorState::Normal;
                            self.first_recovered = None;
                            return Some("hardware-threshold-recovered");
                        }
                    } else {
                        self.first_recovered = Some(Instant::now());
                    }
                } else {
                    self.first_recovered = None;
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TelemetryConfig;
    use std::time::Duration;

    #[test]
    fn test_evaluator_thresholds() {
        let config = TelemetryConfig::default();
        let mut evaluator = HardwareEvaluator::new();

        let mut metrics = SystemMetrics {
            cpu_usage: 90.0, // High CPU
            ram_usage: 50.0,
            gpu_usage: None,
            gpu_mem_usage: None,
        };

        // First tick, should not trigger yet (duration is 15s)
        assert_eq!(evaluator.evaluate(&metrics, &config), None);

        // Simulate time pass
        evaluator.first_exceeded = Some(std::time::Instant::now() - Duration::from_secs(16));

        // Second tick, should trigger
        assert_eq!(
            evaluator.evaluate(&metrics, &config),
            Some("hardware-threshold-exceeded")
        );

        // Now metrics drop
        metrics.cpu_usage = 10.0;
        assert_eq!(evaluator.evaluate(&metrics, &config), None); // not recovered yet, waiting for duration

        evaluator.first_recovered = Some(std::time::Instant::now() - Duration::from_secs(16));
        assert_eq!(
            evaluator.evaluate(&metrics, &config),
            Some("hardware-threshold-recovered")
        );
    }
}
