use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{Patch, PatchParams},
    Api,
};
use serde_json::json;

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
        let current_replicas = dep.spec.as_ref().and_then(|s| s.replicas).unwrap_or(1);

        if current_replicas > 0 {
            let patch = json!({
                "metadata": {
                    "annotations": {
                        "streamk8s.io/previous-replicas": current_replicas.to_string()
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
        let prev_replicas_str = dep
            .metadata
            .annotations
            .as_ref()
            .and_then(|a| a.get("streamk8s.io/previous-replicas"));

        if let Some(prev) = prev_replicas_str {
            if let Ok(count) = prev.parse::<i32>() {
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
    }

    Ok(format!("Namespace {} resumed.", namespace))
}
