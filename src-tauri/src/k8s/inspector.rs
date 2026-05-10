use k8s_openapi::api::core::v1::{Event, Pod};
use kube::{api::ListParams, Api, Client};
use serde_json::Value;

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

// Internal helper for client creation (shared across k8s modules)
async fn create_client(context_name: Option<String>) -> Result<Client, String> {
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
