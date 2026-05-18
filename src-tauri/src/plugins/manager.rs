use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiComponent {
    pub id: String,
    pub r#type: String, // "input", "button", "label"
    pub label: String,
    pub placeholder: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub components: Vec<UiComponent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginManifest {
    pub extension: ExtensionInfo,
    pub ui: UiConfig,
}

#[tauri::command]
pub async fn get_installed_plugins() -> Result<Vec<PluginManifest>, String> {
    Ok(vec![])
}
