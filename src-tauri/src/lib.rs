pub mod hardware;
pub mod k8s;

use crate::hardware::collector::collect_metrics;
use std::time::Duration;
use tauri::Manager;
use sysinfo::System;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![crate::k8s::pods::get_pods])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut sys = System::new_all();
                loop {
                    let metrics = collect_metrics(&mut sys);
                    let _ = handle.emit("hardware-update", metrics);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
