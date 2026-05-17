use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ai_provider: String,
    pub api_key: String,
    pub endpoint: String,
    pub model: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai_provider: "OpenAI".to_string(),
            api_key: "".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            model: "gpt-4-turbo".to_string(),
        }
    }
}

fn get_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut path = app_handle.path().app_config_dir().map_err(|e| e.to_string())?;
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    path.push("settings.json");
    Ok(path)
}

#[tauri::command]
pub async fn get_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    let path = get_config_path(&app_handle)?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn save_config(app_handle: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let path = get_config_path(&app_handle)?;
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}
