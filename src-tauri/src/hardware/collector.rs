use crate::config::TelemetryConfig;
use nvml_wrapper::Nvml;
use serde::Serialize;
use std::time::Instant;
use sysinfo::System;

#[derive(Serialize, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub ram_usage: f32,
    pub gpu_usage: Option<f32>,
    pub gpu_mem_usage: Option<f32>,
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
    state: EvaluatorState,
    first_exceeded: Option<Instant>,
    first_recovered: Option<Instant>,
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

        let (target_state, condition_met, first_timestamp, event_name) = match self.state {
            EvaluatorState::Normal => (
                EvaluatorState::Suspended,
                is_heavy,
                &mut self.first_exceeded,
                "hardware-threshold-exceeded",
            ),
            EvaluatorState::Suspended => (
                EvaluatorState::Normal,
                !is_heavy,
                &mut self.first_recovered,
                "hardware-threshold-recovered",
            ),
        };

        if condition_met {
            let t = first_timestamp.get_or_insert_with(Instant::now);
            if t.elapsed().as_secs() >= config.sustain_duration_seconds as u64 {
                self.state = target_state;
                *first_timestamp = None;
                return Some(event_name);
            }
        } else {
            *first_timestamp = None;
        }

        None
    }

    #[cfg(test)]
    pub fn set_first_exceeded_for_test(&mut self, t: Instant) {
        self.first_exceeded = Some(t);
    }

    #[cfg(test)]
    pub fn set_first_recovered_for_test(&mut self, t: Instant) {
        self.first_recovered = Some(t);
    }
}

pub struct AdaptivePoller {
    idle_start: Option<Instant>,
}

impl AdaptivePoller {
    pub fn new() -> Self {
        Self { idle_start: None }
    }

    pub fn get_interval(&mut self, cpu_usage: f32, has_suspended: bool) -> std::time::Duration {
        if cpu_usage >= 20.0 || has_suspended {
            self.idle_start = None;
            return std::time::Duration::from_secs(1);
        }

        let start = self.idle_start.get_or_insert_with(Instant::now);
        if start.elapsed().as_secs() >= 30 {
            std::time::Duration::from_secs(5)
        } else {
            std::time::Duration::from_secs(1)
        }
    }

    #[cfg(test)]
    pub fn set_idle_start_for_test(&mut self, t: Instant) {
        self.idle_start = Some(t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TelemetryConfig;
    use std::time::Duration;

    #[test]
    fn test_evaluator_thresholds() {
        let config = TelemetryConfig {
            gpu_suspend_threshold: 80,
            cpu_suspend_threshold: 85,
            sustain_duration_seconds: 15,
            ignored_namespaces: Vec::new(),
        };
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
        evaluator.set_first_exceeded_for_test(std::time::Instant::now() - Duration::from_secs(16));

        // Second tick, should trigger
        assert_eq!(
            evaluator.evaluate(&metrics, &config),
            Some("hardware-threshold-exceeded")
        );

        // Now metrics drop
        metrics.cpu_usage = 10.0;
        assert_eq!(evaluator.evaluate(&metrics, &config), None); // not recovered yet, waiting for duration

        evaluator.set_first_recovered_for_test(std::time::Instant::now() - Duration::from_secs(16));
        assert_eq!(
            evaluator.evaluate(&metrics, &config),
            Some("hardware-threshold-recovered")
        );
    }

    #[test]
    fn test_adaptive_poller() {
        let mut poller = AdaptivePoller::new();

        // High CPU -> 1 sec
        assert_eq!(poller.get_interval(25.0, false), Duration::from_secs(1));

        // Low CPU but just started -> 1 sec
        assert_eq!(poller.get_interval(10.0, false), Duration::from_secs(1));

        // Low CPU for 31 secs -> 5 secs
        poller.set_idle_start_for_test(std::time::Instant::now() - Duration::from_secs(31));
        assert_eq!(poller.get_interval(10.0, false), Duration::from_secs(5));

        // Has suspended namespaces -> 1 sec
        assert_eq!(poller.get_interval(10.0, true), Duration::from_secs(1));

        // High CPU again -> 1 sec
        poller.set_idle_start_for_test(std::time::Instant::now() - Duration::from_secs(31));
        assert_eq!(poller.get_interval(25.0, false), Duration::from_secs(1));
    }
}
