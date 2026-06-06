use serde::Serialize;
use kube::Api;
use k8s_openapi::api::core::v1::{Service, Pod};
use k8s_openapi::api::networking::v1::Ingress;
use crate::k8s::inspector::create_client;

#[derive(Serialize, Clone, Debug)]
pub struct TopologyNode {
    pub id: String,
    pub kind: String,
    pub name: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct TopologyEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
}

#[derive(Serialize, Debug)]
pub struct TopologyGraph {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

#[tauri::command]
pub async fn get_namespace_topology(
    context_name: Option<String>,
    namespace: String,
) -> Result<TopologyGraph, String> {
    let client = create_client(context_name).await?;
    
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // 1. Fetch Resources
    let svc_api: Api<Service> = Api::namespaced(client.clone(), &namespace);
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    let ingress_api: Api<Ingress> = Api::namespaced(client.clone(), &namespace);

    let services = svc_api.list(&Default::default()).await.map_err(|e| e.to_string())?;
    let pods = pod_api.list(&Default::default()).await.map_err(|e| e.to_string())?;
    let ingresses = ingress_api.list(&Default::default()).await.map_err(|e| e.to_string())?;

    // 2. Map Services to Nodes and build Selector Map
    for svc in &services {
        let name = svc.metadata.name.clone().unwrap_or_default();
        let id = format!("svc-{}", name);
        nodes.push(TopologyNode {
            id: id.clone(),
            kind: "Service".into(),
            name: name.clone(),
        });

        // Match Pods by Selector
        if let Some(selector) = svc.spec.as_ref().and_then(|s| s.selector.as_ref()) {
            for pod in &pods {
                let pod_labels = pod.metadata.labels.as_ref();
                let pod_name = pod.metadata.name.clone().unwrap_or_default();
                let pod_id = format!("pod-{}", pod_name);

                // Check if all selector labels match pod labels
                let mut matches = true;
                for (key, value) in selector {
                    if pod_labels.and_then(|l| l.get(key)) != Some(value) {
                        matches = false;
                        break;
                    }
                }

                if matches {
                    edges.push(TopologyEdge {
                        id: format!("e-{}-{}", name, pod_name),
                        source: id.clone(),
                        target: pod_id,
                        label: "selects".into(),
                    });
                }
            }
        }
    }

    // 3. Map Pods to Nodes (Only those not already implicit, but let's add all in namespace)
    for pod in &pods {
        let name = pod.metadata.name.clone().unwrap_or_default();
        let id = format!("pod-{}", name);
        // Avoid duplicates if any
        if !nodes.iter().any(|n| n.id == id) {
            nodes.push(TopologyNode {
                id,
                kind: "Pod".into(),
                name,
            });
        }
    }

    // 4. Map Ingresses to Services
    for ing in &ingresses {
        let name = ing.metadata.name.clone().unwrap_or_default();
        let id = format!("ing-{}", name);
        nodes.push(TopologyNode {
            id: id.clone(),
            kind: "Ingress".into(),
            name: name.clone(),
        });

        if let Some(spec) = &ing.spec {
            if let Some(rules) = &spec.rules {
                for rule in rules {
                    if let Some(http) = &rule.http {
                        for path in &http.paths {
                            if let Some(backend_svc) = &path.backend.service {
                                let target_svc_id = format!("svc-{}", backend_svc.name);
                                edges.push(TopologyEdge {
                                    id: format!("e-{}-{}", name, backend_svc.name),
                                    source: id.clone(),
                                    target: target_svc_id,
                                    label: "routes".into(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(TopologyGraph { nodes, edges })
}
