use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{Patch, PatchParams},
    Api,
};
use serde_json::json;

pub struct ActiveClusterState {
    pub context_name: std::sync::Mutex<Option<String>>,
    pub namespace: std::sync::Mutex<String>,
}

#[tauri::command]
pub fn update_active_cluster_state(
    state: tauri::State<'_, ActiveClusterState>,
    context_name: Option<String>,
    namespace: String,
) {
    if let Ok(mut ctx) = state.context_name.lock() {
        *ctx = context_name;
    }
    if let Ok(mut ns) = state.namespace.lock() {
        *ns = namespace;
    }
}

#[tauri::command]
pub async fn scale_workload(
    context_name: Option<String>,
    namespace: String,
    name: String,
    replicas: i32,
) -> Result<(), String> {
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
    let deploy_api: Api<Deployment> = Api::namespaced(client, &namespace);

    let patch = json!({
        "spec": { "replicas": replicas }
    });

    deploy_api
        .patch(
            &name,
            &PatchParams::apply("streamk8s"),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn suspend_namespace(
    context_name: Option<String>,
    namespace: String,
) -> Result<String, String> {
    let client = crate::k8s::inspector::create_client(context_name).await?;
    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let deployments = api
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;

    for dep in deployments.items {
        let name = dep.metadata.name.unwrap_or_default();
        let current_replicas = dep.spec.as_ref().and_then(|s| s.replicas);

        if should_suspend(current_replicas) {
            let replicas_to_store = current_replicas.unwrap_or(1);
            let patch = json!({
                "metadata": {
                    "annotations": {
                        "streamk8s.io/previous-replicas": replicas_to_store.to_string()
                    }
                },
                "spec": {
                    "replicas": 0
                }
            });

            api.patch(
                &name,
                &PatchParams::apply("streamk8s"),
                &Patch::Apply(&patch),
            )
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(format!("Namespace {} suspended.", namespace))
}

#[tauri::command]
pub async fn resume_namespace(
    context_name: Option<String>,
    namespace: String,
) -> Result<String, String> {
    let client = crate::k8s::inspector::create_client(context_name).await?;
    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let deployments = api
        .list(&Default::default())
        .await
        .map_err(|e| e.to_string())?;

    for dep in deployments.items {
        let name = dep.metadata.name.unwrap_or_default();
        if let Some(count) = get_previous_replicas(&dep.metadata.annotations) {
            let patch = json!({
                "spec": {
                    "replicas": count
                }
            });

            // Using merge patch for specific field update
            api.patch(&name, &PatchParams::default(), &Patch::Merge(&patch))
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(format!("Namespace {} resumed.", namespace))
}

fn get_previous_replicas(
    annotations: &Option<std::collections::BTreeMap<String, String>>,
) -> Option<i32> {
    annotations
        .as_ref()
        .and_then(|a| a.get("streamk8s.io/previous-replicas"))
        .and_then(|prev| prev.parse::<i32>().ok())
}

fn should_suspend(replicas: Option<i32>) -> bool {
    replicas.unwrap_or(1) > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_get_previous_replicas() {
        let mut annotations = BTreeMap::new();
        annotations.insert(
            "streamk8s.io/previous-replicas".to_string(),
            "3".to_string(),
        );
        assert_eq!(get_previous_replicas(&Some(annotations)), Some(3));

        assert_eq!(get_previous_replicas(&None), None);

        let mut annotations2 = BTreeMap::new();
        annotations2.insert(
            "streamk8s.io/previous-replicas".to_string(),
            "invalid".to_string(),
        );
        assert_eq!(get_previous_replicas(&Some(annotations2)), None);
    }

    #[test]
    fn test_should_suspend() {
        assert!(should_suspend(Some(1)));
        assert!(should_suspend(Some(5)));
        assert!(!should_suspend(Some(0)));
        assert!(should_suspend(None)); // Default to 1, so should suspend
    }
}
