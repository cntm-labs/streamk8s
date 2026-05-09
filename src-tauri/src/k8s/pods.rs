use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use serde::Serialize;

#[derive(Serialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
}

#[tauri::command]
pub async fn get_pods(context_name: Option<String>) -> Result<Vec<PodInfo>, String> {
    let config = if let Some(name) = context_name {
        let mut options = kube::config::KubeConfigOptions::default();
        options.context = Some(name);
        kube::Config::from_kubeconfig(&options)
            .await
            .map_err(|e| e.to_string())?
    } else {
        kube::Config::infer().await.map_err(|e| e.to_string())?
    };
    let client = kube::Client::try_from(config).map_err(|e| e.to_string())?;
    let pods: Api<Pod> = Api::all(client);
    let lp = pods
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;

    Ok(lp
        .items
        .into_iter()
        .map(|p| PodInfo {
            name: p.metadata.name.unwrap_or_default(),
            namespace: p.metadata.namespace.unwrap_or_default(),
            status: p
                .status
                .and_then(|s| s.phase)
                .unwrap_or_else(|| "Unknown".to_string()),
        })
        .collect())
}
