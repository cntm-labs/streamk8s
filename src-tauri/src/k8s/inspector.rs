use k8s_openapi::api::core::v1::{Event, Pod};
use kube::{
    api::ListParams,
    api::{Patch, PatchParams},
    Api, Client,
};
use serde_json::Value;
use tokio::io::AsyncReadExt;

#[tauri::command]
pub async fn apply_resource_manifest(
    context_name: Option<String>,
    namespace: String,
    pod_name: String,
    yaml_content: String,
) -> Result<(), String> {
    let client = create_client(context_name).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    let patch: Pod = serde_yaml::from_str(&yaml_content).map_err(|e| e.to_string())?;

    pods.patch(
        &pod_name,
        &PatchParams::apply("streamk8s"),
        &Patch::Apply(&patch),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_resource_manifest(
    context_name: Option<String>,
    namespace: String,
    pod_name: String,
) -> Result<String, String> {
    let client = create_client(context_name).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    let p = pods.get(&pod_name).await.map_err(|e| e.to_string())?;
    serde_yaml::to_string(&p).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_pod_events(
    context_name: Option<String>,
    namespace: String,
    pod_name: String,
) -> Result<Value, String> {
    let client = create_client(context_name).await?;
    let events: Api<Event> = Api::namespaced(client, &namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={}", pod_name));
    let evs = events.list(&lp).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(evs).unwrap_or(serde_json::json!([])))
}

#[tauri::command]
pub async fn read_pod_file(
    context_name: Option<String>,
    namespace: String,
    pod_name: String,
    container_name: String,
    file_path: String,
) -> Result<String, String> {
    let client = create_client(context_name).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    let command = vec!["cat", &file_path];

    let mut attached = pods
        .exec(
            &pod_name,
            command,
            &kube::api::AttachParams::default()
                .container(&container_name)
                .stdout(true),
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut output = String::new();
    attached
        .stdout()
        .unwrap()
        .read_to_string(&mut output)
        .await
        .map_err(|e| e.to_string())?;
    Ok(output)
}

#[tauri::command]
pub async fn write_pod_file(
    context_name: Option<String>,
    namespace: String,
    pod_name: String,
    container_name: String,
    file_path: String,
    content_base64: String,
) -> Result<(), String> {
    let client = create_client(context_name).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    // Command to write via base64 to avoid shell escaping issues
    let script = format!("echo {} | base64 -d > {}", content_base64, file_path);
    let command = vec!["sh", "-c", &script];

    pods.exec(
        &pod_name,
        command,
        &kube::api::AttachParams::default().container(&container_name),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Internal helper for client creation (shared across k8s modules)
pub(crate) async fn create_client(context_name: Option<String>) -> Result<Client, String> {
    let config = if let Some(name) = context_name {
        let mut options = kube::config::KubeConfigOptions::default();
        options.context = Some(name);
        kube::Config::from_kubeconfig(&options)
            .await
            .map_err(|e| e.to_string())?
    } else {
        kube::Config::infer().await.map_err(|e| e.to_string())?
    };
    Client::try_from(config).map_err(|e| e.to_string())
}
