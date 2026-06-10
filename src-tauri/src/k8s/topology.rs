use crate::k8s::inspector::create_client;
use k8s_openapi::api::core::v1::{Node, Pod, Service};
use k8s_openapi::api::networking::v1::Ingress;
use kube::Api;
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct TopologyNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub parent_id: Option<String>,
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

    // 0. Fetch Nodes
    let node_api: Api<Node> = Api::all(client.clone());
    let hw_nodes = node_api
        .list(&Default::default())
        .await
        .map(|l| l.items)
        .unwrap_or_default();

    // 1. Fetch Pods (Core requirement)
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    let pods = pod_api
        .list(&Default::default())
        .await
        .map(|l| l.items)
        .unwrap_or_default();

    // 2. Fetch Services
    let svc_api: Api<Service> = Api::namespaced(client.clone(), &namespace);
    let services = svc_api
        .list(&Default::default())
        .await
        .map(|l| l.items)
        .unwrap_or_default();

    // 3. Fetch Ingresses (Optional, might not exist in all clusters)
    let ingress_api: Api<Ingress> = Api::namespaced(client.clone(), &namespace);
    let ingresses = ingress_api
        .list(&Default::default())
        .await
        .map(|l| l.items)
        .unwrap_or_default();

    // Map Nodes to Nodes (TopologyNodes)
    for hw_node in &hw_nodes {
        let name = hw_node.metadata.name.clone().unwrap_or_default();
        let id = format!("node-{}", name);
        nodes.push(TopologyNode {
            id,
            kind: "Node".into(),
            name,
            parent_id: None,
        });
    }

    // Map Services to Nodes and build Selector Map
    for svc in &services {
        let name = svc.metadata.name.clone().unwrap_or_default();
        let id = format!("svc-{}", name);
        nodes.push(TopologyNode {
            id: id.clone(),
            kind: "Service".into(),
            name: name.clone(),
            parent_id: None,
        });

        if let Some(selector) = svc.spec.as_ref().and_then(|s| s.selector.as_ref()) {
            for pod in &pods {
                let pod_labels = pod.metadata.labels.as_ref();
                let pod_name = pod.metadata.name.clone().unwrap_or_default();
                let pod_id = format!("pod-{}", pod_name);

                let mut matches = true;
                if let Some(p_labels) = pod_labels {
                    for (key, value) in selector {
                        if p_labels.get(key) != Some(value) {
                            matches = false;
                            break;
                        }
                    }
                } else {
                    matches = false;
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

    // Map Pods to Nodes
    for pod in &pods {
        let name = pod.metadata.name.clone().unwrap_or_default();
        let id = format!("pod-{}", name);
        let parent_id = pod
            .spec
            .as_ref()
            .and_then(|s| s.node_name.clone())
            .map(|n| format!("node-{}", n));

        if !nodes.iter().any(|n| n.id == id) {
            nodes.push(TopologyNode {
                id,
                kind: "Pod".into(),
                name,
                parent_id,
            });
        }
    }

    // Map Ingresses to Services
    for ing in &ingresses {
        let name = ing.metadata.name.clone().unwrap_or_default();
        let id = format!("ing-{}", name);
        nodes.push(TopologyNode {
            id: id.clone(),
            kind: "Ingress".into(),
            name: name.clone(),
            parent_id: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_node_serialization_with_parent() {
        let node = TopologyNode {
            id: "1".into(),
            kind: "Pod".into(),
            name: "test-pod".into(),
            parent_id: Some("node-1".into()),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains(r#""parent_id":"node-1""#));
    }
}
