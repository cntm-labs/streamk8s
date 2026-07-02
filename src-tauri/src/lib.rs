pub mod config;
pub mod hardware;
pub mod k8s;
pub mod plugins;


use crate::hardware::collector::collect_metrics;

use crate::k8s::scaling::{update_active_cluster_state, ActiveClusterState};
use nvml_wrapper::Nvml;

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
        .manage(ActiveClusterState {
            context_name: std::sync::Mutex::new(None),
            namespace: std::sync::Mutex::new("default".to_string()),
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
            crate::k8s::inspector::list_pod_files,
            crate::k8s::inspector::delete_k8s_resource,
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
            crate::k8s::terminal::close_terminal_session,
            update_active_cluster_state
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let nvml = Nvml::init().ok();
            tauri::async_runtime::spawn(async move {
                let mut sys = System::new_all();
                let mut evaluator = crate::hardware::collector::HardwareEvaluator::new();
                loop {
                    let metrics = collect_metrics(&mut sys, &nvml);
                    let _ = handle.emit("hardware-update", &metrics);

                    // Read configuration
                    let config = crate::config::AppConfig::load(&handle).unwrap_or_default();

                    // Only evaluate if auto_suspend is globally enabled
                    if config.auto_suspend {
                        if let Some(event) = evaluator.evaluate(&metrics, &config.telemetry) {
                            if event == "hardware-threshold-exceeded" {
                                println!("Threshold exceeded, triggering auto-suspend...");
                                let _ = handle.emit("hardware-threshold-exceeded", "Sustained Heavy Load");
                            } else if event == "hardware-threshold-recovered" {
                                println!("Threshold recovered, resuming workloads...");
                                let _ = handle.emit("hardware-threshold-recovered", "Load normalized");
                            }
                        }
                    }

                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
