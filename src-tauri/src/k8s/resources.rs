use kube::Api;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Service, ConfigMap, Secret};
use serde::Serialize;
use super::inspector::create_client;

#[derive(Serialize)]
pub struct NormalizedResource {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub kind: String,
}

#[tauri::command]
pub async fn get_deployments(context_name: Option<String>, namespace: String) -> Result<Vec<NormalizedResource>, String> {
    let client = create_client(context_name).await?;
    let api: Api<Deployment> = Api::namespaced(client, &namespace);
    let list = api.list(&Default::default()).await.map_err(|e| e.to_string())?;
    Ok(list.items.into_iter().map(|d| NormalizedResource {
        name: d.metadata.name.clone().unwrap_or_default(),
        namespace: d.metadata.namespace.clone().unwrap_or_default(),
        status: format!("{}/{}", 
            d.status.as_ref().and_then(|s| s.available_replicas).unwrap_or(0),
            d.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0)
        ),
        kind: "Deployment".to_string(),
    }).collect())
}

#[tauri::command]
pub async fn get_services(context_name: Option<String>, namespace: String) -> Result<Vec<NormalizedResource>, String> {
    let client = create_client(context_name).await?;
    let api: Api<Service> = Api::namespaced(client, &namespace);
    let list = api.list(&Default::default()).await.map_err(|e| e.to_string())?;
    Ok(list.items.into_iter().map(|s| NormalizedResource {
        name: s.metadata.name.clone().unwrap_or_default(),
        namespace: s.metadata.namespace.clone().unwrap_or_default(),
        status: s.spec.as_ref().and_then(|spec| spec.type_.clone()).unwrap_or_else(|| "ClusterIP".to_string()),
        kind: "Service".to_string(),
    }).collect())
}

#[tauri::command]
pub async fn get_configmaps(context_name: Option<String>, namespace: String) -> Result<Vec<NormalizedResource>, String> {
    let client = create_client(context_name).await?;
    let api: Api<ConfigMap> = Api::namespaced(client, &namespace);
    let list = api.list(&Default::default()).await.map_err(|e| e.to_string())?;
    Ok(list.items.into_iter().map(|c| NormalizedResource {
        name: c.metadata.name.clone().unwrap_or_default(),
        namespace: c.metadata.namespace.clone().unwrap_or_default(),
        status: format!("{} items", c.data.as_ref().map(|d| d.len()).unwrap_or(0)),
        kind: "ConfigMap".to_string(),
    }).collect())
}

#[tauri::command]
pub async fn get_secrets(context_name: Option<String>, namespace: String) -> Result<Vec<NormalizedResource>, String> {
    let client = create_client(context_name).await?;
    let api: Api<Secret> = Api::namespaced(client, &namespace);
    let list = api.list(&Default::default()).await.map_err(|e| e.to_string())?;
    Ok(list.items.into_iter().map(|s| NormalizedResource {
        name: s.metadata.name.clone().unwrap_or_default(),
        namespace: s.metadata.namespace.clone().unwrap_or_default(),
        status: s.type_.clone().unwrap_or_else(|| "Opaque".to_string()),
        kind: "Secret".to_string(),
    }).collect())
}
