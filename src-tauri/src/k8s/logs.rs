use futures_util::io::AsyncBufReadExt;
use futures_util::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::LogParams, Api};
use std::sync::Mutex;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter};

// Static to keep track of the active log stream task to prevent multiple tasks
static ACTIVE_STREAM: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

#[tauri::command]
pub async fn start_log_stream(
    handle: AppHandle,
    context_name: Option<String>,
    namespace: String,
    pod_name: String,
) -> Result<(), String> {
    // Abort previous stream if any
    {
        let mut active_stream = ACTIVE_STREAM.lock().unwrap();
        if let Some(handle) = active_stream.take() {
            handle.abort();
        }
    }

    let config = if let Some(name) = context_name {
        let mut options = kube::config::KubeConfigOptions::default();
        options.context = Some(name.clone());
        kube::Config::from_kubeconfig(&options).await.map_err(|e| {
            let err = format!("Failed to load kubeconfig for context {}: {}", name, e);
            eprintln!("{}", err);
            err
        })?
    } else {
        kube::Config::infer().await.map_err(|e| {
            let err = format!("Failed to infer kubeconfig: {}", e);
            eprintln!("{}", err);
            err
        })?
    };

    let client = kube::Client::try_from(config).map_err(|e| {
        let err = format!("Failed to create K8s client: {}", e);
        eprintln!("{}", err);
        err
    })?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    let mut lp = LogParams::default();
    lp.follow = true;
    lp.tail_lines = Some(200); // Increased for better initial experience

    let stream = pods.log_stream(&pod_name, &lp).await.map_err(|e| {
        let err = format!("Failed to start log stream for {}: {}", pod_name, e);
        eprintln!("{}", err);
        err
    })?;

    let new_handle = tauri::async_runtime::spawn(async move {
        let mut lines = stream.lines();
        while let Some(line) = lines.next().await {
            match line {
                Ok(line_str) => {
                    if let Err(e) = handle.emit("pod-log-line", line_str) {
                        eprintln!("Failed to emit log line: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error in log stream for pod {}: {}", pod_name, e);
                    break;
                }
            }
        }
        eprintln!("Log stream ended for pod {}", pod_name);
    });

    // Store the new handle
    {
        let mut active_stream = ACTIVE_STREAM.lock().unwrap();
        *active_stream = Some(new_handle);
    }

    Ok(())
}
