use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{Patch, PatchParams},
    Api, Client,
};
use serde_json::json;

#[tauri::command]
pub async fn scale_workload(namespace: String, name: String, replicas: i32) -> Result<(), String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
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
