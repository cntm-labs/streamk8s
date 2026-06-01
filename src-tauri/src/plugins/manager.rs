use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Module, Store, Linker, Caller};
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::io::Cursor;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemotePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub url: String,
    pub category: String,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryContent {
    pub version: String,
    pub plugins: Vec<RemotePlugin>,
}

fn get_plugin_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/streamk8s/plugins")
}

fn get_registry_cache_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/streamk8s/registry_cache.json")
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

#[tauri::command]
pub async fn get_remote_registry() -> Result<Vec<RemotePlugin>, String> {
    let url = "https://raw.githubusercontent.com/cntm-labs/streamk8s/master/registry.json";
    let cache_path = get_registry_cache_path();

    match reqwest::get(url).await {
        Ok(response) => {
            if response.status().is_success() {
                let content = response.text().await.map_err(|e| e.to_string())?;
                let registry: RegistryContent = serde_json::from_str(&content).map_err(|e| e.to_string())?;
                
                if let Some(parent) = cache_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::write(&cache_path, &content).ok();
                
                return Ok(registry.plugins);
            }
        }
        Err(_) => {
            if cache_path.exists() {
                let content = fs::read_to_string(&cache_path).map_err(|e| e.to_string())?;
                let registry: RegistryContent = serde_json::from_str(&content).map_err(|e| e.to_string())?;
                return Ok(registry.plugins);
            }
        }
    }

    Ok(Vec::new())
}

#[tauri::command]
pub async fn install_remote_plugin(id: String, url: String) -> Result<(), String> {
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let plugin_dir = get_plugin_dir().join(&id);
    if !plugin_dir.exists() {
        fs::create_dir_all(&plugin_dir).map_err(|e| e.to_string())?;
    }

    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => plugin_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn install_plugin(source_path: String) -> Result<(), String> {
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err("Source path does not exist".to_string());
    }

    let plugin_id = source.file_name().ok_or("Invalid source path")?.to_str().unwrap();
    let dest = get_plugin_dir().join(plugin_id);
    
    if !dest.exists() {
        fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    }

    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let dest_file = dest.join(entry.file_name());
        fs::copy(entry.path(), dest_file).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn create_linker(engine: &Engine) -> Linker<()> {
    let mut linker = Linker::new(engine);
    
    linker.func_wrap("env", "get_k8s_resources_count", |_: Caller<'_, ()>| -> i32 {
        42
    }).unwrap();

    linker.func_wrap("env", "show_notification", |code: i32| {
        println!("PLUGIN NOTIFICATION CODE: {}", code);
    }).unwrap();

    linker
}

pub fn execute_wasm_action(
    wasm_bytes: &[u8],
    function_name: &str,
    _payload_json: &str,
) -> Result<String, String> {
    let engine = Engine::default();
    let module = Module::from_binary(&engine, wasm_bytes).map_err(|e| e.to_string())?;
    
    let mut store = Store::new(&engine, ());
    let linker = create_linker(&engine);
    
    let instance = linker.instantiate(&mut store, &module).map_err(|e| e.to_string())?;

    let func = instance
        .get_typed_func::<(), ()>(&mut store, function_name)
        .map_err(|e| format!("Function '{}' not found or wrong signature: {}", function_name, e))?;

    func.call(&mut store, ()).map_err(|e| e.to_string())?;

    Ok(format!("Action '{}' executed successfully via WASM ABI.", function_name))
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
