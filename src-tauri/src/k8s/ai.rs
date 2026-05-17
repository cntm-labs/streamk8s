use crate::config::AppConfig;
use crate::k8s::inspector::get_pod_events;
use crate::k8s::inspector::get_resource_manifest;
use serde_json::json;

trait AiProvider {
    async fn analyze(&self, prompt: &str, config: &AppConfig) -> Result<String, String>;
}

struct OpenAiProvider;

impl AiProvider for OpenAiProvider {
    async fn analyze(&self, prompt: &str, config: &AppConfig) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", config.endpoint.trim_end_matches('/'));
        
        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&json!({
                "model": config.model,
                "messages": [
                    {"role": "system", "content": "You are a Kubernetes expert. Summarize the issue and provide fix steps in markdown format. Be concise."},
                    {"role": "user", "content": prompt}
                ]
            }))
            .send()
            .await
            .map_err(|e| format!("OpenAI Request Error: {}", e))?;

        let json: serde_json::Value = res.json().await.map_err(|e| format!("OpenAI JSON Error: {}", e))?;

        if let Some(err) = json["error"]["message"].as_str() {
            return Err(format!("AI Error: {}", err));
        }

        Ok(json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("No advice found")
            .to_string())
    }
}

struct OllamaProvider;

impl AiProvider for OllamaProvider {
    async fn analyze(&self, prompt: &str, config: &AppConfig) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/chat", config.endpoint.trim_end_matches('/'));
        
        let res = client.post(&url)
            .json(&json!({
                "model": config.model,
                "messages": [
                    {"role": "system", "content": "You are a Kubernetes expert. Summarize the issue and provide fix steps in markdown format. Be concise."},
                    {"role": "user", "content": prompt}
                ],
                "stream": false
            }))
            .send()
            .await
            .map_err(|e| format!("Ollama Request Error: {}", e))?;

        let json: serde_json::Value = res.json().await.map_err(|e| format!("Ollama JSON Error: {}", e))?;

        if let Some(err) = json["error"].as_str() {
            return Err(format!("Ollama Error: {}", err));
        }

        Ok(json["message"]["content"]
            .as_str()
            .unwrap_or("No advice found")
            .to_string())
    }
}

#[tauri::command]
pub async fn analyze_with_ai(
    app_handle: tauri::AppHandle,
    context_name: Option<String>,
    namespace: String,
    resource_name: String,
) -> Result<String, String> {
    // 1. Fetch Config
    let config = crate::config::get_config(app_handle).await?;

    // 2. Aggregate Context
    let manifest = get_resource_manifest(
        context_name.clone(),
        namespace.clone(),
        resource_name.clone(),
    )
    .await?;
    let events = get_pod_events(context_name, namespace, resource_name).await?;

    let prompt = format!(
        "Analyze the following Kubernetes Resource state and provide a diagnostic report.\n\nMANIFEST:\n{}\n\nEVENTS:\n{:?}",
        manifest, events
    );

    // 3. Call Provider directly based on config
    match config.ai_provider.to_lowercase().as_str() {
        "ollama" => OllamaProvider.analyze(&prompt, &config).await,
        _ => OpenAiProvider.analyze(&prompt, &config).await, // Default to OpenAI
    }
}
