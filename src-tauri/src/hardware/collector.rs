use serde::Serialize;
use sysinfo::System;

#[derive(Serialize, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub ram_usage: f32,
    pub gpu_usage: Option<f32>,
}

pub fn collect_metrics(sys: &mut System) -> SystemMetrics {
    // In sysinfo 0.30, traits like SystemExt and CpuExt are no longer needed.
    // Methods are now implemented directly on the types.
    sys.refresh_all();

    let cpu = sys.global_cpu_info().cpu_usage();
    let ram = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;

    SystemMetrics {
        cpu_usage: cpu,
        ram_usage: ram,
        gpu_usage: None,
    }
}
