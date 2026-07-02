use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub gpu_suspend_threshold: u32,
    pub cpu_suspend_threshold: u32,
    pub sustain_duration_seconds: u32,
    #[serde(default)]
    pub ignored_namespaces: Vec<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            gpu_suspend_threshold: 80,
            cpu_suspend_threshold: 85,
            sustain_duration_seconds: 15,
            ignored_namespaces: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ai_provider: String,
    pub api_key: String,
    pub endpoint: String,
    pub model: String,
    #[serde(default)]
    pub auto_suspend: bool,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai_provider: "OpenAI".to_string(),
            api_key: "".to_string(),
            endpoint: "https://api.openai.com/v1".to_string(),
            model: "gpt-4-turbo".to_string(),
            auto_suspend: false,
            telemetry: TelemetryConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn load(app_handle: &tauri::AppHandle) -> Result<Self, String> {
        let path = get_config_path(app_handle)?;
        if !path.exists() {
            return Ok(AppConfig::default());
        }
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(config)
    }
}

fn get_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut path = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    path.push("settings.json");
    Ok(path)
}

#[tauri::command]
pub async fn get_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    AppConfig::load(&app_handle)
}

#[tauri::command]
pub async fn save_config(app_handle: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let path = get_config_path(&app_handle)?;
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.telemetry.gpu_suspend_threshold, 80);
        assert_eq!(config.telemetry.cpu_suspend_threshold, 85);
        assert_eq!(config.telemetry.sustain_duration_seconds, 15);
    }

    #[test]
    fn test_telemetry_config_ignored_namespaces() {
        let config = TelemetryConfig::default();
        assert_eq!(config.ignored_namespaces, Vec::<String>::new());
    }
}
