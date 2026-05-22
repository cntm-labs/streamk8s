use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Module, Store, Linker};
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
    let engine = Engine::default();
    let module = Module::from_binary(&engine, wasm_bytes).map_err(|e| e.to_string())?;
    
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    
    // In Milestone 12, we provide no host imports.
    // Plugins must be self-contained or use standard WASI (not yet added).
    let instance = linker.instantiate(&mut store, &module).map_err(|e| e.to_string())?;

    let func = instance
        .get_typed_func::<(), ()>(&mut store, function_name)
        .map_err(|e| format!("Function '{}' not found or has wrong signature: {}", function_name, e))?;

    func.call(&mut store, ()).map_err(|e| e.to_string())?;

    Ok(format!("Function '{}' executed successfully.", function_name))
}

#[tauri::command]
pub async fn call_plugin_action(
    plugin_id: String,
    action_name: String,
    payload: String,
) -> Result<String, String> {
    let wasm_path = get_plugin_dir().join(&plugin_id).join("logic.wasm");
    
    if !wasm_path.exists() {
        return Ok(format!("Mock action '{}' executed for plugin '{}' with payload: {}", action_name, plugin_id, payload));
    }

    let wasm_bytes = std::fs::read(&wasm_path).map_err(|e| e.to_string())?;
    execute_wasm_action(&wasm_bytes, &action_name, &payload)
}
