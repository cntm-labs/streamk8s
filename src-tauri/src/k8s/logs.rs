use futures_util::io::AsyncBufReadExt;
use futures_util::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::LogParams, Api, Client};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn start_log_stream(
    handle: AppHandle,
    namespace: String,
    pod_name: String,
) -> Result<(), String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    let mut lp = LogParams::default();
    lp.follow = true;
    lp.tail_lines = Some(100);

    let stream = pods
        .log_stream(&pod_name, &lp)
        .await
        .map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn(async move {
        let mut lines = stream.lines();
        while let Some(line) = lines.next().await {
            if let Ok(line_str) = line {
                let _ = handle.emit("pod-log-line", line_str);
            }
        }
    });

    Ok(())
}
