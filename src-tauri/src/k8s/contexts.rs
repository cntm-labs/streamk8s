use kube::config::Kubeconfig;
use serde::Serialize;

#[derive(Serialize)]
pub struct ClusterContext {
    pub name: String,
    pub is_current: bool,
}

#[tauri::command]
pub async fn get_available_contexts() -> Result<Vec<ClusterContext>, String> {
    // Read the default Kubeconfig
    let config = Kubeconfig::read().map_err(|e| e.to_string())?;
    let current_context = config.current_context.clone();
    
    Ok(config.contexts.into_iter().map(|c| ClusterContext {
        name: c.name.clone(),
        is_current: current_context.as_ref() == Some(&c.name),
    }).collect())
}
