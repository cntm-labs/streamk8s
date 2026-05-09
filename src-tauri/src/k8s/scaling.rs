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
