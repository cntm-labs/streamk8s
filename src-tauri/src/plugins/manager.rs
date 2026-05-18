use serde::{Deserialize, Serialize};
use wasmer::{imports, Instance, Module, Store};
use std::fs;
use std::path::PathBuf;

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

fn get_plugin_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/streamk8s/plugins")
}

#[tauri::command]
pub async fn get_installed_plugins() -> Result<Vec<PluginManifest>, String> {
    let plugin_dir = get_plugin_dir();
    if !plugin_dir.exists() {
        fs::create_dir_all(&plugin_dir).map_err(|e| e.to_string())?;
    }

    let mut plugins = Vec::new();
    let entries = fs::read_dir(plugin_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("extension.toml");
            if manifest_path.exists() {
                let content = fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
                let manifest: PluginManifest = toml::from_str(&content).map_err(|e| e.to_string())?;
                plugins.push(manifest);
            }
        }
    }

    Ok(plugins)
}

pub fn execute_wasm_action(
    wasm_bytes: &[u8],
    function_name: &str,
    _payload_json: &str,
) -> Result<String, String> {
    let mut store = Store::default();
    let module = Module::new(&store, wasm_bytes).map_err(|e| e.to_string())?;

    // Define a basic import object (empty for now, will expand in future milestones)
    let import_object = imports! {};

    let instance = Instance::new(&mut store, &module, &import_object).map_err(|e| e.to_string())?;

    let func = instance
        .exports
        .get_function(function_name)
        .map_err(|e| e.to_string())?;

    // For simplicity in Milestone 12, we assume the function takes no args or we pass one string pointer
    // Real implementation would use a more complex ABI.
    // Let's just try to call a basic "init" or "run" function for now.

    let result = func.call(&mut store, &[]).map_err(|e| e.to_string())?;

    Ok(format!("Function executed. Result: {:?}", result))
}

#[tauri::command]
pub async fn call_plugin_action(
    plugin_id: String,
    action_name: String,
    payload: String,
) -> Result<String, String> {
    let wasm_path = get_plugin_dir().join(&plugin_id).join("logic.wasm");
    
    // For now, if WASM doesn't exist, return a mock success for the dummy plugin
    if !wasm_path.exists() {
        return Ok(format!("Mock action '{}' executed for plugin '{}' with payload: {}", action_name, plugin_id, payload));
    }

    let wasm_bytes = std::fs::read(&wasm_path).map_err(|e| e.to_string())?;
    execute_wasm_action(&wasm_bytes, &action_name, &payload)
}
