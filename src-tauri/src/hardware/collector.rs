use nvml_wrapper::Nvml;
use serde::Serialize;
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
