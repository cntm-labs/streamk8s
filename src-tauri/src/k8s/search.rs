use serde::Serialize;
use kube::config::Kubeconfig;

#[derive(Serialize, Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub kind: String, // 'Cluster', 'Resource', or 'Action'
    pub context: Option<String>,
    pub namespace: Option<String>,
}

#[tauri::command]
pub async fn global_search(query: String) -> Result<Vec<SearchResult>, String> {
    let mut results = vec![
        SearchResult {
            id: "action_settings".into(),
            title: "Open Settings".into(),
            subtitle: "Application Configuration".into(),
            kind: "Action".into(),
            context: None,
            namespace: None,
        },
        SearchResult {
            id: "action_explorer".into(),
            title: "Switch to Explorer".into(),
            subtitle: "Kubernetes Resources".into(),
            kind: "Action".into(),
            context: None,
            namespace: None,
        },
        SearchResult {
            id: "action_hardware".into(),
            title: "Show Telemetry".into(),
            subtitle: "Hardware Stats".into(),
            kind: "Action".into(),
            context: None,
            namespace: None,
        },
    ];

    // Load kubeconfig to get contexts
    if let Ok(config) = Kubeconfig::read() {
        for ctx in config.contexts {
            results.push(SearchResult {
                id: format!("ctx_{}", ctx.name),
                title: ctx.name.clone(),
                subtitle: "Kubernetes Context".into(),
                kind: "Cluster".into(),
                context: Some(ctx.name),
                namespace: None,
            });
        }
    }

    if !query.is_empty() {
        let q = query.to_lowercase();
        results.retain(|r| {
            r.title.to_lowercase().contains(&q) || r.subtitle.to_lowercase().contains(&q)
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_global_search_includes_contexts() {
        let results = global_search("".to_string()).await.unwrap();
        let cluster_exists = results.iter().any(|r| r.kind == "Cluster");
        assert!(cluster_exists, "Should find at least one cluster context if kubeconfig exists");
    }

    #[tokio::test]
    async fn test_global_search_actions() {
        let results = global_search("settings".to_string()).await.unwrap();
        assert!(results.iter().any(|r| r.id == "action_settings"));
    }
}
