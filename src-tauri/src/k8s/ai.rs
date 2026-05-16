use serde_json::json;
use crate::k8s::inspector::get_resource_manifest;
use crate::k8s::inspector::get_pod_events;

#[tauri::command]
pub async fn analyze_with_ai(
    context_name: Option<String>,
    namespace: String,
    resource_name: String,
    api_key: String,
) -> Result<String, String> {
    // 1. Aggregate Context
    let manifest = get_resource_manifest(context_name.clone(), namespace.clone(), resource_name.clone()).await?;
    let events = get_pod_events(context_name, namespace, resource_name).await?;
    
    let prompt = format!(
        "Analyze the following Kubernetes Resource state and provide a diagnostic report.\n\nMANIFEST:\n{}\n\nEVENTS:\n{:?}",
        manifest, events
    );

    // 2. Call OpenAI-compatible API
    let client = reqwest::Client::new();
    let res = client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "gpt-4-turbo",
            "messages": [
                {"role": "system", "content": "You are a Kubernetes expert. Summarize the issue and provide fix steps in markdown format. Be concise."},
                {"role": "user", "content": prompt}
            ]
        }))
        .send().await.map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    
    if let Some(err) = json["error"]["message"].as_str() {
        return Err(format!("AI Error: {}", err));
    }

    Ok(json["choices"][0]["message"]["content"].as_str().unwrap_or("No advice found").to_string())
}
