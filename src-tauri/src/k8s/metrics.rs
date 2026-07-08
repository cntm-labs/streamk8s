use kube::{
    api::{Api, ApiResource, ListParams},
    core::DynamicObject,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PodMetricsResult {
    pub name: String,
    pub namespace: String,
    pub cpu: String,
    pub memory: String,
}

#[tauri::command]
pub async fn get_pod_metrics(
    context_name: Option<String>,
    namespace: String,
) -> Result<Vec<PodMetricsResult>, String> {
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

    let gvk = kube::api::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
    let api_resource = ApiResource::from_gvk(&gvk);

    let metrics_api: Api<DynamicObject> = if namespace.is_empty() || namespace == "all" {
        Api::all_with(client, &api_resource)
    } else {
        Api::namespaced_with(client, &namespace, &api_resource)
    };

    let metrics_list = match metrics_api.list(&ListParams::default()).await {
        Ok(l) => l,
        Err(e) => return Err(format!("Failed to list pod metrics: {}", e)),
    };

    let mut result = Vec::new();

    for item in metrics_list.items {
        let name = item.metadata.name.clone().unwrap_or_default();
        let ns = item.metadata.namespace.clone().unwrap_or_default();

        let mut total_cpu = 0u64;
        let mut total_mem = 0u64;

        if let Some(containers) = item.data.get("containers").and_then(|c| c.as_array()) {
            for container in containers {
                if let Some(usage) = container.get("usage") {
                    if let Some(cpu) = usage.get("cpu").and_then(|c| c.as_str()) {
                        total_cpu += parse_cpu(cpu);
                    }
                    if let Some(mem) = usage.get("memory").and_then(|m| m.as_str()) {
                        total_mem += parse_memory(mem);
                    }
                }
            }
        }

        result.push(PodMetricsResult {
            name,
            namespace: ns,
            cpu: format!("{}m", total_cpu / 1_000_000), // convert nano to milli
            memory: format!("{}Mi", total_mem / 1024 / 1024), // convert bytes to MiB
        });
    }

    Ok(result)
}

fn parse_cpu(cpu: &str) -> u64 {
    if cpu.ends_with('n') {
        cpu[..cpu.len() - 1].parse().unwrap_or(0)
    } else if cpu.ends_with('u') {
        cpu[..cpu.len() - 1].parse::<u64>().unwrap_or(0) * 1_000
    } else if cpu.ends_with('m') {
        cpu[..cpu.len() - 1].parse::<u64>().unwrap_or(0) * 1_000_000
    } else {
        cpu.parse::<f64>().unwrap_or(0.0) as u64 * 1_000_000_000
    }
}

fn parse_memory(mem: &str) -> u64 {
    if mem.ends_with("Ki") {
        mem[..mem.len() - 2].parse::<u64>().unwrap_or(0) * 1024
    } else if mem.ends_with("Mi") {
        mem[..mem.len() - 2].parse::<u64>().unwrap_or(0) * 1024 * 1024
    } else if mem.ends_with("Gi") {
        mem[..mem.len() - 2].parse::<u64>().unwrap_or(0) * 1024 * 1024 * 1024
    } else if mem.ends_with("Ti") {
        mem[..mem.len() - 2].parse::<u64>().unwrap_or(0) * 1024 * 1024 * 1024 * 1024
    } else {
        mem.parse().unwrap_or(0)
    }
}
