use crate::config::AppConfig;
use crate::k8s::inspector::get_pod_events;
use serde::{Deserialize, Serialize};
use serde_json::json;

trait AiProvider {
    async fn analyze(&self, prompt: &str, config: &AppConfig) -> Result<String, String>;
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
}

struct GeminiProvider;

impl AiProvider for GeminiProvider {
    async fn analyze(&self, prompt: &str, config: &AppConfig) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            config.model, config.api_key
        );

        let system_prompt = "You are a Senior Kubernetes Site Reliability Engineer (SRE). Your task is to perform a deep root-cause analysis on the provided resource manifest and events.
- Identify hidden misconfigurations (e.g., resource limits, probe timings).
- Cross-reference events with manifest states.
- Provide a 'Root Cause' and a 'Recommended Action Plan' in professional markdown.
- Include a specific YAML snippet or kubectl command for the fix.";

        let full_prompt = format!("{}\n\nUser Request: {}", system_prompt, prompt);

        let res = client
            .post(&url)
            .json(&GeminiRequest {
                contents: vec![GeminiContent {
                    parts: vec![GeminiPart { text: full_prompt }],
                }],
            })
            .send()
            .await
            .map_err(|e| format!("Gemini Request Error: {}", e))?;

        let json: GeminiResponse = res
            .json()
            .await
            .map_err(|e| format!("Gemini JSON Error: {}", e))?;

        if let Some(err) = json.error {
            return Err(format!("Gemini AI Error: {}", err.message));
        }

        if let Some(candidates) = json.candidates {
            if let Some(candidate) = candidates.get(0) {
                if let Some(part) = candidate.content.parts.get(0) {
                    return Ok(part.text.clone());
                }
            }
        }

        Err("No advice found from Gemini".to_string())
    }
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
                    {"role": "system", "content": "You are a Senior Kubernetes Site Reliability Engineer (SRE). Your task is to perform a deep root-cause analysis on the provided resource manifest and events.\n- Identify hidden misconfigurations (e.g., resource limits, probe timings).\n- Cross-reference events with manifest states.\n- Provide a 'Root Cause' and a 'Recommended Action Plan' in professional markdown.\n- Include a specific YAML snippet or kubectl command for the fix."},
                    {"role": "user", "content": prompt}
                ]
            }))
            .send()
            .await
            .map_err(|e| format!("OpenAI Request Error: {}", e))?;

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| format!("OpenAI JSON Error: {}", e))?;

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
                    {"role": "system", "content": "You are a Senior Kubernetes Site Reliability Engineer (SRE). Your task is to perform a deep root-cause analysis on the provided resource manifest and events.\n- Identify hidden misconfigurations (e.g., resource limits, probe timings).\n- Cross-reference events with manifest states.\n- Provide a 'Root Cause' and a 'Recommended Action Plan' in professional markdown.\n- Include a specific YAML snippet or kubectl command for the fix."},
                    {"role": "user", "content": prompt}
                ],
                "stream": false
            }))
            .send()
            .await
            .map_err(|e| format!("Ollama Request Error: {}", e))?;

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| format!("Ollama JSON Error: {}", e))?;

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
    kind: String,
    namespace: String,
    resource_name: String,
) -> Result<String, String> {
    // 1. Fetch Config
    let config = crate::config::get_config(app_handle).await?;

    // 2. Aggregate Context
    let manifest = crate::k8s::resources::get_k8s_resource_details(
        context_name.clone(),
        kind,
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
        "gemini" => GeminiProvider.analyze(&prompt, &config).await,
        "claude" => OpenAiProvider.analyze(&prompt, &config).await, // Use OpenAI proxy for Claude
        _ => OpenAiProvider.analyze(&prompt, &config).await,        // Default to OpenAI
    }
}
