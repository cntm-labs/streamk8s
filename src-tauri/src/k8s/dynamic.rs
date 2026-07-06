use crate::k8s::inspector::create_client;
use kube::api::{Api, DynamicObject};
use kube::discovery::Discovery;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct DynamicResourceInfo {
    pub group: String,
    pub version: String,
    pub kind: String,
    pub plural: String,
    pub namespaced: bool,
}

#[tauri::command]
pub async fn get_api_resources(
    context_name: Option<String>,
) -> Result<HashMap<String, Vec<DynamicResourceInfo>>, String> {
    let client = create_client(context_name).await?;
    let discovery = Discovery::new(client)
        .run()
        .await
        .map_err(|e| e.to_string())?;

    let mut result: HashMap<String, Vec<DynamicResourceInfo>> = HashMap::new();

    for group in discovery.groups() {
        let group_name = if group.name().is_empty() {
            "core".to_string()
        } else {
            group.name().to_string()
        };

        for (ar, caps) in group.recommended_resources() {
            if caps.supports_operation(kube::discovery::verbs::LIST)
                && caps.supports_operation(kube::discovery::verbs::GET)
            {
                let info = DynamicResourceInfo {
                    group: ar.group.clone(),
                    version: ar.version.clone(),
                    kind: ar.kind.clone(),
                    plural: ar.plural.clone(),
                    namespaced: caps.scope == kube::discovery::Scope::Namespaced,
                };
                result.entry(group_name.clone()).or_default().push(info);
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn list_dynamic_resource(
    context_name: Option<String>,
    group: String,
    version: String,
    kind: String,
    plural: String,
    namespaced: bool,
    namespace: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = create_client(context_name).await?;

    let ar = kube::api::ApiResource {
        group: group.clone(),
        version: version.clone(),
        api_version: if group.is_empty() {
            version.clone()
        } else {
            format!("{}/{}", group, version)
        },
        kind: kind.clone(),
        plural: plural.clone(),
    };

    let api: Api<DynamicObject> = if namespaced {
        if let Some(ns) = namespace {
            if !ns.is_empty() && ns != "All Namespaces" {
                Api::namespaced_with(client, &ns, &ar)
            } else {
                Api::all_with(client, &ar)
            }
        } else {
            Api::all_with(client, &ar)
        }
    } else {
        Api::all_with(client, &ar)
    };

    let list = api
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;
    let items = list
        .items
        .into_iter()
        .map(|item| serde_json::to_value(item).unwrap_or(serde_json::Value::Null))
        .collect();

    Ok(items)
}
