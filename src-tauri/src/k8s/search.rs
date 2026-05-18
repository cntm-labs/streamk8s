use serde::Serialize;

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
    // Basic action registry
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

    // Filter results based on query (case-insensitive)
    if !query.is_empty() {
        let q = query.to_lowercase();
        results.retain(|r| {
            r.title.to_lowercase().contains(&q) || r.subtitle.to_lowercase().contains(&q)
        });
    }

    Ok(results)
}
