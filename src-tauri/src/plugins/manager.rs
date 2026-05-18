use serde::{Deserialize, Serialize};
use wasmer::{imports, Instance, Module, Store};

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
    wasm_path: String,
    function_name: String,
    payload_json: String,
) -> Result<String, String> {
    let wasm_bytes = std::fs::read(&wasm_path).map_err(|e| e.to_string())?;
    execute_wasm_action(&wasm_bytes, &function_name, &payload_json)
}
