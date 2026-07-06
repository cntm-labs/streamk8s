use k8s_openapi::api::core::v1::Pod;
use kube::api::Api;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::copy_bidirectional;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Clone, Serialize, Deserialize)]
pub struct ForwardSession {
    pub id: String,
    pub context: String,
    pub namespace: String,
    pub pod: String,
    pub local_port: u16,
    pub remote_port: u16,
}

pub struct PortForwardManager {
    // Map of session ID to ForwardSession details
    pub sessions: Arc<Mutex<HashMap<String, ForwardSession>>>,
    // Map of session ID to the JoinHandle of the listener loop, and a cancellation sender
    pub handles: Arc<Mutex<HashMap<String, (JoinHandle<()>, oneshot::Sender<()>)>>>,
}

impl PortForwardManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tauri::command]
pub async fn start_port_forward(
    manager: tauri::State<'_, PortForwardManager>,
    context: String,
    namespace: String,
    pod: String,
    local_port: u16,
    remote_port: u16,
) -> Result<String, String> {
    let client = crate::k8s::inspector::create_client(Some(context.clone()))
        .await
        .map_err(|e| e.to_string())?;

    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    // Bind local port
    let bind_addr = format!("127.0.0.1:{}", local_port);
    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| format!("Failed to bind local port {}: {}", local_port, e))?;

    let session_id = uuid::Uuid::new_v4().to_string();
    let session = ForwardSession {
        id: session_id.clone(),
        context: context.clone(),
        namespace: namespace.clone(),
        pod: pod.clone(),
        local_port,
        remote_port,
    };

    let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut cancel_rx => {
                    break;
                }
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((mut client_stream, _)) => {
                            let pods = pods.clone();
                            let pod = pod.clone();

                            tokio::spawn(async move {
                                let mut pf = match pods.portforward(&pod, &[remote_port]).await {
                                    Ok(pf) => pf,
                                    Err(e) => {
                                        eprintln!("Error starting portforward to pod {}: {}", pod, e);
                                        return;
                                    }
                                };

                                let mut upstream = match pf.take_stream(remote_port) {
                                    Some(stream) => stream,
                                    None => {
                                        eprintln!("Failed to take stream for remote port {}", remote_port);
                                        return;
                                    }
                                };

                                if let Err(e) = copy_bidirectional(&mut client_stream, &mut upstream).await {
                                    eprintln!("Error copying data for port forward: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Error accepting tcp connection: {}", e);
                        }
                    }
                }
            }
        }
    });

    manager
        .sessions
        .lock()
        .await
        .insert(session_id.clone(), session);
    manager
        .handles
        .lock()
        .await
        .insert(session_id.clone(), (handle, cancel_tx));

    Ok(session_id)
}

#[tauri::command]
pub async fn stop_port_forward(
    manager: tauri::State<'_, PortForwardManager>,
    session_id: String,
) -> Result<(), String> {
    let mut handles = manager.handles.lock().await;
    if let Some((handle, cancel_tx)) = handles.remove(&session_id) {
        let _ = cancel_tx.send(());
        handle.abort();
    } else {
        return Err("Session not found".to_string());
    }

    manager.sessions.lock().await.remove(&session_id);
    Ok(())
}

#[tauri::command]
pub async fn list_active_forwards(
    manager: tauri::State<'_, PortForwardManager>,
) -> Result<Vec<ForwardSession>, String> {
    let sessions = manager.sessions.lock().await;
    Ok(sessions.values().cloned().collect())
}
