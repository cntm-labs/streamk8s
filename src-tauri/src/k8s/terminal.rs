// src-tauri/src/k8s/terminal.rs

use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::Sender;
use tokio::task::AbortHandle;

pub struct TerminalSession {
    pub stdin_tx: Sender<Vec<u8>>,
    pub writer_abort: AbortHandle,
    pub reader_abort: AbortHandle,
}

pub struct TerminalSessionManager {
    pub sessions: Mutex<HashMap<String, TerminalSession>>,
}

#[tauri::command]
pub async fn start_terminal_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, TerminalSessionManager>,
    context_name: Option<String>,
    namespace: String,
    pod_name: String,
    container_name: String,
    session_id: String,
) -> Result<(), String> {
    let client = crate::k8s::inspector::create_client(context_name).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    // Check bash first, fallback to sh
    let mut attached = match pods
        .exec(
            &pod_name,
            vec!["/bin/bash"],
            &AttachParams::default()
                .container(&container_name)
                .stdin(true)
                .stdout(true)
                .stderr(true)
                .tty(true),
        )
        .await
    {
        Ok(res) => res,
        Err(_) => pods
            .exec(
                &pod_name,
                vec!["/bin/sh"],
                &AttachParams::default()
                    .container(&container_name)
                    .stdin(true)
                    .stdout(true)
                    .stderr(true)
                    .tty(true),
            )
            .await
            .map_err(|e| format!("Both bash and sh failed: {}", e))?,
    };

    let mut stdout = attached.stdout().ok_or("Failed to attach to stdout")?;
    let mut stdin = attached.stdin().ok_or("Failed to attach to stdin")?;

    let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);

    // Writer Task (stdin)
    let writer_task = tokio::spawn(async move {
        while let Some(data) = stdin_rx.recv().await {
            if stdin.write_all(&data).await.is_err() {
                break;
            }
            let _ = stdin.flush().await;
        }
    });

    // Reader Task (stdout)
    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let reader_task = tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        loop {
            match stdout.read(&mut buffer).await {
                Ok(0) => {
                    let _ = app_clone.emit(&format!("terminal-exit-{}", session_id_clone), ());
                    break;
                }
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buffer[..n]).to_string();
                    let _ = app_clone.emit(&format!("terminal-stdout-{}", session_id_clone), text);
                }
                Err(_) => {
                    let _ = app_clone.emit(&format!("terminal-exit-{}", session_id_clone), ());
                    break;
                }
            }
        }
    });

    let session = TerminalSession {
        stdin_tx,
        writer_abort: writer_task.abort_handle(),
        reader_abort: reader_task.abort_handle(),
    };

    state.sessions.lock().unwrap().insert(session_id, session);
    Ok(())
}

#[tauri::command]
pub async fn send_terminal_input(
    state: tauri::State<'_, TerminalSessionManager>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let stdin_tx = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&session_id).map(|s| s.stdin_tx.clone())
    };
    if let Some(tx) = stdin_tx {
        let _ = tx.send(data.into_bytes()).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn close_terminal_session(
    state: tauri::State<'_, TerminalSessionManager>,
    session_id: String,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().unwrap();
    if let Some(session) = sessions.remove(&session_id) {
        session.writer_abort.abort();
        session.reader_abort.abort();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_registration() {
        let manager = TerminalSessionManager {
            sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
        };
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        let writer_task = tokio::spawn(async {});
        let reader_task = tokio::spawn(async {});

        let session = TerminalSession {
            stdin_tx: tx,
            writer_abort: writer_task.abort_handle(),
            reader_abort: reader_task.abort_handle(),
        };

        manager
            .sessions
            .lock()
            .unwrap()
            .insert("test-session".to_string(), session);
        assert!(manager
            .sessions
            .lock()
            .unwrap()
            .contains_key("test-session"));

        manager.sessions.lock().unwrap().remove("test-session");
        assert!(!manager
            .sessions
            .lock()
            .unwrap()
            .contains_key("test-session"));
    }

    #[tokio::test]
    async fn test_mock_terminal_write() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        tx.send(b"hello".to_vec()).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, b"hello");
    }
}
