pub mod hardware;
pub mod k8s;

use crate::hardware::collector::collect_metrics;
use crate::hardware::profiler::Profiler;
use nvml_wrapper::Nvml;
use serde_json::json;
use std::time::Duration;
use sysinfo::System;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            crate::k8s::pods::get_pods,
            crate::k8s::scaling::scale_workload,
            crate::k8s::logs::start_log_stream,
            crate::k8s::contexts::get_available_contexts,
            crate::k8s::inspector::get_resource_manifest,
            crate::k8s::inspector::apply_resource_manifest,
            crate::k8s::inspector::get_pod_events,
            crate::k8s::inspector::read_pod_file,
            crate::k8s::inspector::write_pod_file
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let nvml = Nvml::init().ok();
            tauri::async_runtime::spawn(async move {
                let mut sys = System::new_all();
                loop {
                    let metrics = collect_metrics(&mut sys, &nvml);
                    let _ = handle.emit("hardware-update", &metrics);

                    let heavy_apps = Profiler::scan_heavy_apps(&sys);
                    if !heavy_apps.is_empty()
                        || metrics.cpu_usage > 80.0
                        || metrics.gpu_usage.unwrap_or(0.0) > 80.0
                    {
                        let _ = handle.emit(
                            "smart-advice",
                            json!({
                                "action": "Suspend",
                                "reason": if !heavy_apps.is_empty() {
                                    format!("Heavy apps detected: {:?}", heavy_apps)
                                } else {
                                    "High system load detected".to_string()
                                },
                                "suggested_pods": ["*"]
                            }),
                        );
                    }

                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
