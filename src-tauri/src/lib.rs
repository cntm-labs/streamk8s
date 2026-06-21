pub mod config;
pub mod hardware;
pub mod k8s;
pub mod plugins;

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
        .plugin(tauri_plugin_dialog::init())
        .manage(crate::k8s::terminal::TerminalSessionManager {
            sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            crate::config::get_config,
            crate::config::save_config,
            crate::k8s::pods::get_pods,
            crate::k8s::scaling::scale_workload,
            crate::k8s::scaling::suspend_namespace,
            crate::k8s::scaling::resume_namespace,
            crate::k8s::logs::start_log_stream,
            crate::k8s::contexts::get_available_contexts,
            crate::k8s::inspector::apply_resource_manifest,
            crate::k8s::inspector::apply_k8s_resource,
            crate::k8s::inspector::get_pod_events,
            crate::k8s::inspector::read_pod_file,
            crate::k8s::inspector::write_pod_file,
            crate::k8s::resources::get_deployments,
            crate::k8s::resources::get_services,
            crate::k8s::resources::get_configmaps,
            crate::k8s::resources::get_secrets,
            crate::k8s::resources::get_k8s_resource_details,
            crate::k8s::resources::get_namespaces,
            crate::k8s::ai::analyze_with_ai,
            crate::k8s::search::global_search,
            crate::k8s::topology::get_namespace_topology,
            crate::plugins::manager::get_installed_plugins,
            crate::plugins::manager::call_plugin_action,
            crate::plugins::manager::install_plugin,
            crate::plugins::manager::get_remote_registry,
            crate::k8s::terminal::start_terminal_session,
            crate::k8s::terminal::send_terminal_input,
            crate::k8s::terminal::close_terminal_session
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let nvml = Nvml::init().ok();
            tauri::async_runtime::spawn(async move {
                let mut sys = System::new_all();
                loop {
                    let metrics = collect_metrics(&mut sys, &nvml);
                    crate::hardware::collector::check_for_heavy_apps(&sys, &handle);
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
