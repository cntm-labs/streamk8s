use k8s_openapi::api::core::v1::{Event, Pod};
use kube::core::DynamicObject;
use kube::discovery::{Discovery, Scope};
use kube::{
    api::ListParams,
    api::{Patch, PatchParams},
    Api, Client,
};
use serde_json::Value;
use tokio::io::AsyncReadExt;

#[tauri::command]
pub async fn apply_k8s_resource(
    context_name: Option<String>,
    kind: String,
    namespace: String,
    name: String,
    yaml: String,
    dry_run: bool,
) -> Result<String, String> {
    let client = create_client(context_name).await?;
    let discovery =
        crate::k8s::retry::with_retry(|| async { Discovery::new(client.clone()).run().await })
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
            let api: kube::Api<DynamicObject> = if caps.scope == Scope::Namespaced {
                kube::Api::namespaced_with(client.clone(), &namespace, &ar)
            } else {
                kube::Api::all_with(client.clone(), &ar)
            };

            let patch: DynamicObject =
                serde_yaml::from_str(&yaml).map_err(|e| format!("Invalid YAML: {}", e))?;

            let mut pp = PatchParams::apply("streamk8s");
            pp.force = true;
            if dry_run {
                pp.dry_run = true;
            }

            let _obj = crate::k8s::retry::with_retry(|| async {
                api.patch(&name, &pp, &Patch::Apply(&patch)).await
            })
            .await
            .map_err(|e| e.to_string())?;
            return Ok(format!("Successfully applied {} {}", kind, name));
        }
    }

    Err(format!("Resource kind '{}' not found", kind))
}

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

    crate::k8s::retry::with_retry(|| async {
        pods.patch(
            &pod_name,
            &PatchParams::apply("streamk8s"),
            &Patch::Apply(&patch),
        )
        .await
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
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
    let evs = crate::k8s::retry::with_retry(|| async { events.list(&lp).await })
        .await
        .map_err(|e| e.to_string())?;
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
    let mut attached = crate::k8s::retry::with_retry(|| async {
        let command = vec!["cat", file_path.as_str()];
        pods.exec(
            &pod_name,
            command,
            &kube::api::AttachParams::default()
                .container(&container_name)
                .stdout(true),
        )
        .await
    })
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
    crate::k8s::retry::with_retry(|| async {
        let command = vec!["sh", "-c", script.as_str()];
        pods.exec(
            &pod_name,
            command,
            &kube::api::AttachParams::default().container(&container_name),
        )
        .await
    })
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

#[tauri::command]
pub async fn list_pod_files(
    context_name: Option<String>,
    namespace: String,
    pod_name: String,
    container_name: String,
    dir_path: String,
) -> Result<serde_json::Value, String> {
    let client = create_client(context_name).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    // Command to list files. Appending -F shows / for directories.
    let mut attached = crate::k8s::retry::with_retry(|| async {
        let command = vec!["ls", "-F", "--color=never", dir_path.as_str()];
        pods.exec(
            &pod_name,
            command,
            &kube::api::AttachParams::default()
                .container(&container_name)
                .stdout(true),
        )
        .await
    })
    .await
    .map_err(|e| e.to_string())?;

    let mut output = String::new();
    attached
        .stdout()
        .unwrap()
        .read_to_string(&mut output)
        .await
        .map_err(|e| e.to_string())?;

    let mut files = Vec::new();
    for line in output.lines() {
        let name = line.trim();
        if name.is_empty() {
            continue;
        }
        let is_dir = name.ends_with('/');
        let clean_name = if is_dir {
            &name[..name.len() - 1]
        } else {
            name
        };
        files.push(serde_json::json!({
            "name": clean_name,
            "is_dir": is_dir
        }));
    }

    Ok(serde_json::json!(files))
}

#[tauri::command]
pub async fn delete_k8s_resource(
    context_name: Option<String>,
    kind: String,
    namespace: String,
    name: String,
) -> Result<String, String> {
    let client = create_client(context_name).await?;
    let discovery =
        crate::k8s::retry::with_retry(|| async { Discovery::new(client.clone()).run().await })
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
            let api: kube::Api<DynamicObject> = if caps.scope == Scope::Namespaced {
                kube::Api::namespaced_with(client.clone(), &namespace, &ar)
            } else {
                kube::Api::all_with(client.clone(), &ar)
            };

            let dp = kube::api::DeleteParams::default();
            crate::k8s::retry::with_retry(|| async { api.delete(&name, &dp).await })
                .await
                .map_err(|e| e.to_string())?;
            return Ok(format!("Successfully deleted {} {}", kind, name));
        }
    }

    Err(format!("Resource kind '{}' not found", kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_params_compiles() {
        let dp = kube::api::DeleteParams::default();
        assert_eq!(dp.grace_period_seconds, None);
    }

    #[test]
    fn test_parse_ls_output() {
        let sample_output = "bin/\netc/\nhosts\nresolv.conf\n";
        let mut files = Vec::new();
        for line in sample_output.lines() {
            let name = line.trim();
            if name.is_empty() {
                continue;
            }
            let is_dir = name.ends_with('/');
            let clean_name = if is_dir {
                &name[..name.len() - 1]
            } else {
                name
            };
            files.push(serde_json::json!({
                "name": clean_name,
                "is_dir": is_dir
            }));
        }

        assert_eq!(files.len(), 4);
        assert_eq!(files[0]["name"], "bin");
        assert_eq!(files[0]["is_dir"], true);
        assert_eq!(files[2]["name"], "hosts");
        assert_eq!(files[2]["is_dir"], false);
    }
}
