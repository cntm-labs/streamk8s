use serde::Serialize;

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
}

#[derive(Serialize, Debug)]
pub struct TopologyGraph {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

#[tauri::command]
pub async fn get_namespace_topology(
    _context_name: String,
    _namespace: String,
) -> Result<TopologyGraph, String> {
    // Return a mock graph for now to unblock frontend development.
    Ok(TopologyGraph {
        nodes: vec![
            TopologyNode {
                id: "ingress-1".into(),
                kind: "Ingress".into(),
                name: "api-gateway".into(),
            },
            TopologyNode {
                id: "svc-1".into(),
                kind: "Service".into(),
                name: "web-svc".into(),
            },
            TopologyNode {
                id: "pod-1".into(),
                kind: "Pod".into(),
                name: "web-pod-a".into(),
            },
            TopologyNode {
                id: "pod-2".into(),
                kind: "Pod".into(),
                name: "web-pod-b".into(),
            },
        ],
        edges: vec![
            TopologyEdge {
                id: "e1".into(),
                source: "ingress-1".into(),
                target: "svc-1".into(),
            },
            TopologyEdge {
                id: "e2".into(),
                source: "svc-1".into(),
                target: "pod-1".into(),
            },
            TopologyEdge {
                id: "e3".into(),
                source: "svc-1".into(),
                target: "pod-2".into(),
            },
        ],
    })
}
