pub mod config;
pub mod hardware;
pub mod k8s;
pub mod plugins;
pub mod state_manager;

use crate::hardware::collector::collect_metrics;

use crate::k8s::scaling::{update_active_cluster_state, ActiveClusterState};
use nvml_wrapper::Nvml;

use sysinfo::System;
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

pub struct SuspendedStateCache(pub std::sync::Mutex<std::collections::HashSet<String>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(crate::k8s::terminal::TerminalSessionManager {
            sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
        .manage(crate::k8s::portforward::PortForwardManager::new())
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
            crate::k8s::dynamic::get_api_resources,
            crate::k8s::dynamic::list_dynamic_resource,
            crate::k8s::portforward::start_port_forward,
            crate::k8s::portforward::stop_port_forward,
            crate::k8s::portforward::list_active_forwards,
            update_active_cluster_state
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let app_data_dir = handle.path().app_data_dir().ok();
            let state_manager = crate::state_manager::StateManager::new(app_data_dir);
            let state = state_manager.load();
            if !state.suspended_namespaces.is_empty() {
                println!(
                    "Recovered suspended namespaces from persistent state: {:?}",
                    state.suspended_namespaces
                );
            }
            app.manage(SuspendedStateCache(std::sync::Mutex::new(
                state.suspended_namespaces,
            )));

            let (nvml, permission_error) = match Nvml::init() {
                Ok(n) => (Some(n), None),
                Err(e) => (None, Some(format!("GPU monitoring requires Admin/Root or NVIDIA drivers are missing. ({})", e))),
            };
            tauri::async_runtime::spawn(async move {
                let mut sys = System::new_all();
                let mut evaluator = crate::hardware::collector::HardwareEvaluator::new();
                let mut adaptive_poller = crate::hardware::collector::AdaptivePoller::new();

                loop {
                    let metrics = collect_metrics(&mut sys, &nvml, &permission_error);
                    let _ = handle.emit("hardware-update", &metrics);

                    // Read configuration
                    let config = crate::config::AppConfig::load(&handle).unwrap_or_default();

                    let has_suspended = {
                        let cache = handle.state::<SuspendedStateCache>();
                        let is_empty = cache.0.lock().unwrap().is_empty();
                        !is_empty
                    };

                    let interval = adaptive_poller.get_interval(metrics.cpu_usage, has_suspended);

                    // Only evaluate if auto_suspend is globally enabled
                    if config.auto_suspend {
                        if let Some(event) = evaluator.evaluate(&metrics, &config.telemetry) {
                            if event == "hardware-threshold-exceeded" {
                                println!("Threshold exceeded, triggering auto-suspend...");
                                let _ = handle
                                    .emit("hardware-threshold-exceeded", "Sustained Heavy Load");

                                let context_name = {
                                    let active_state = handle.state::<ActiveClusterState>();
                                    let name = active_state.context_name.lock().unwrap().clone();
                                    name
                                };

                                let handle_clone = handle.clone();
                                let config_clone = config.clone();

                                tauri::async_runtime::spawn(async move {
                                    if let Ok(namespaces) =
                                        crate::k8s::resources::get_namespaces(context_name.clone())
                                            .await
                                    {
                                        let cache = handle_clone.state::<SuspendedStateCache>();
                                        let mut suspended_count = 0;
                                        for ns in namespaces {
                                            if !crate::k8s::scaling::is_namespace_ignored(
                                                &ns,
                                                &config_clone,
                                            ) {
                                                if crate::k8s::scaling::suspend_namespace(
                                                    handle_clone.clone(),
                                                    cache.clone(),
                                                    context_name.clone(),
                                                    ns,
                                                )
                                                .await
                                                .is_ok()
                                                {
                                                    suspended_count += 1;
                                                }
                                            }
                                        }
                                        if suspended_count > 0 {
                                            let _ = handle_clone
                                                .notification()
                                                .builder()
                                                .title("StreamK8s: Resource Optimization")
                                                .body(format!(
                                                    "High load detected. Auto-suspended {} namespaces.",
                                                    suspended_count
                                                ))
                                                .show();
                                        }
                                    }
                                });
                            } else if event == "hardware-threshold-recovered" {
                                println!("Threshold recovered, resuming workloads...");
                                let _ =
                                    handle.emit("hardware-threshold-recovered", "Load normalized");

                                let context_name = {
                                    let active_state = handle.state::<ActiveClusterState>();
                                    let name = active_state.context_name.lock().unwrap().clone();
                                    name
                                };

                                let handle_clone = handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    let namespaces_to_resume: Vec<String> = {
                                        let cache = handle_clone.state::<SuspendedStateCache>();
                                        let list: Vec<String> =
                                            cache.0.lock().unwrap().iter().cloned().collect();
                                        list
                                    };
                                    let cache = handle_clone.state::<SuspendedStateCache>();
                                    let mut resumed_count = 0;
                                    for ns in namespaces_to_resume {
                                        if crate::k8s::scaling::resume_namespace(
                                            handle_clone.clone(),
                                            cache.clone(),
                                            context_name.clone(),
                                            ns,
                                        )
                                        .await
                                        .is_ok()
                                        {
                                            resumed_count += 1;
                                        }
                                    }
                                    if resumed_count > 0 {
                                        let _ = handle_clone
                                            .notification()
                                            .builder()
                                            .title("StreamK8s: Normal Operation")
                                            .body(format!(
                                                "System load normalized. Resumed {} namespaces.",
                                                resumed_count
                                            ))
                                            .show();
                                    }
                                });
                            }
                        }
                    }

                    tokio::time::sleep(interval).await;
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
