use super::inspector::create_client;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Secret, Service};
use kube::api::{Api, DynamicObject};
use kube::discovery::{Discovery, Scope};
use serde::Serialize;

#[derive(Serialize)]
pub struct NormalizedResource {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub kind: String,
}

#[tauri::command]
pub async fn get_namespaces(context_name: Option<String>) -> Result<Vec<String>, String> {
    let client = create_client(context_name).await?;
    let ns_api: Api<k8s_openapi::api::core::v1::Namespace> = Api::all(client);
    let list = ns_api
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;
    Ok(list
        .items
        .into_iter()
        .filter_map(|ns| ns.metadata.name)
        .collect())
}

#[tauri::command]
pub async fn get_k8s_resource_details(
    context_name: Option<String>,
    kind: String,
    namespace: String,
    name: String,
) -> Result<String, String> {
    let client = create_client(context_name).await?;
    let discovery = Discovery::new(client.clone())
        .run()
        .await
        .map_err(|e| e.to_string())?;

    let normalized_kind = match kind.as_str() {
        "Pods" => "Pod",
        "Deployments" => "Deployment",
        "Services" => "Service",
        "ConfigMaps" => "ConfigMap",
        "Secrets" => "Secret",
        "Namespaces" => "Namespace",
        "Nodes" => "Node",
        "StatefulSets" => "StatefulSet",
        "DaemonSets" => "DaemonSet",
        "Ingresses" => "Ingress",
        _ => {
            if kind.ends_with('s') && kind.len() > 1 {
                &kind[..kind.len() - 1]
            } else {
                &kind
            }
        }
    };

    for group in discovery.groups() {
        if let Some((ar, caps)) = group.recommended_kind(normalized_kind) {
            let api: Api<DynamicObject> = if caps.scope == Scope::Namespaced {
                Api::namespaced_with(client.clone(), &namespace, &ar)
            } else {
                Api::all_with(client.clone(), &ar)
            };

            let obj = api.get(&name).await.map_err(|e| e.to_string())?;
            return serde_yaml::to_string(&obj).map_err(|e| e.to_string());
        }
    }

    Err(format!("Resource kind '{}' not found", kind))
}

#[tauri::command]
pub async fn get_deployments(
    context_name: Option<String>,
    namespace: String,
) -> Result<Vec<NormalizedResource>, String> {
    let client = create_client(context_name).await?;
    let api: Api<Deployment> = Api::namespaced(client, &namespace);
    let list = api
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;
    Ok(list
        .items
        .into_iter()
        .map(|d| NormalizedResource {
            name: d.metadata.name.clone().unwrap_or_default(),
            namespace: d.metadata.namespace.clone().unwrap_or_default(),
            status: format!(
                "{}/{}",
                d.status
                    .as_ref()
                    .and_then(|s| s.available_replicas)
                    .unwrap_or(0),
                d.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0)
            ),
            kind: "Deployment".to_string(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_services(
    context_name: Option<String>,
    namespace: String,
) -> Result<Vec<NormalizedResource>, String> {
    let client = create_client(context_name).await?;
    let api: Api<Service> = Api::namespaced(client, &namespace);
    let list = api
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;
    Ok(list
        .items
        .into_iter()
        .map(|s| NormalizedResource {
            name: s.metadata.name.clone().unwrap_or_default(),
            namespace: s.metadata.namespace.clone().unwrap_or_default(),
            status: s
                .spec
                .as_ref()
                .and_then(|spec| spec.type_.clone())
                .unwrap_or_else(|| "ClusterIP".to_string()),
            kind: "Service".to_string(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_configmaps(
    context_name: Option<String>,
    namespace: String,
) -> Result<Vec<NormalizedResource>, String> {
    let client = create_client(context_name).await?;
    let api: Api<ConfigMap> = Api::namespaced(client, &namespace);
    let list = api
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;
    Ok(list
        .items
        .into_iter()
        .map(|c| NormalizedResource {
            name: c.metadata.name.clone().unwrap_or_default(),
            namespace: c.metadata.namespace.clone().unwrap_or_default(),
            status: format!("{} items", c.data.as_ref().map(|d| d.len()).unwrap_or(0)),
            kind: "ConfigMap".to_string(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_secrets(
    context_name: Option<String>,
    namespace: String,
) -> Result<Vec<NormalizedResource>, String> {
    let client = create_client(context_name).await?;
    let api: Api<Secret> = Api::namespaced(client, &namespace);
    let list = api
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;
    Ok(list
        .items
        .into_iter()
        .map(|s| NormalizedResource {
            name: s.metadata.name.clone().unwrap_or_default(),
            namespace: s.metadata.namespace.clone().unwrap_or_default(),
            status: s.type_.clone().unwrap_or_else(|| "Opaque".to_string()),
            kind: "Secret".to_string(),
        })
        .collect())
}
