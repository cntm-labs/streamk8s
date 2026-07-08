use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
    pub revision: String,
    pub updated: String,
    pub status: String,
    pub chart: String,
    pub app_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelmHistory {
    pub revision: i32,
    pub updated: String,
    pub status: String,
    pub chart: String,
    pub app_version: String,
    pub description: String,
}

#[command]
pub async fn list_helm_releases(namespace: Option<String>) -> Result<Vec<HelmRelease>, String> {
    let mut cmd = Command::new("helm");
    cmd.arg("list").arg("-o").arg("json");

    if let Some(ns) = namespace {
        if !ns.is_empty() {
            cmd.arg("--namespace").arg(ns);
        } else {
            cmd.arg("--all-namespaces");
        }
    } else {
        cmd.arg("--all-namespaces");
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute helm: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Helm command failed: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let releases: Vec<HelmRelease> =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(releases)
}

#[command]
pub async fn get_helm_release_history(
    name: String,
    namespace: String,
) -> Result<Vec<HelmHistory>, String> {
    let mut cmd = Command::new("helm");
    cmd.arg("history").arg(&name).arg("-o").arg("json");

    if !namespace.is_empty() {
        cmd.arg("--namespace").arg(namespace);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute helm: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Helm command failed: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let history: Vec<HelmHistory> =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(history)
}
