use tauri::Manager;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct ChatRequest {
    #[serde(rename = "baseUrl")]
    base_url: String,
    #[serde(rename = "apiKey")]
    api_key: String,
    model: String,
    temperature: f64,
    messages: Vec<Value>,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    content: String,
}

#[tauri::command]
fn chat_completions(request: ChatRequest) -> Result<ChatResponse, String> {
    let base_url = request.base_url.trim_end_matches('/');
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|error| error.to_string())?;

    let payload = serde_json::json!({
        "model": request.model,
        "temperature": request.temperature,
        "stream": false,
        "messages": request.messages,
    });

    let response = client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(request.api_key)
        .json(&payload)
        .send()
        .map_err(|error| format!("网络请求失败：{error}"))?;

    let status = response.status();
    let body: Value = response
        .json()
        .map_err(|error| format!("响应解析失败：{error}"))?;

    if !status.is_success() {
        return Err(body.to_string());
    }

    let content = body
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    if content.is_empty() {
        Err(format!("模型没有返回内容：{body}"))
    } else {
        Ok(ChatResponse { content })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![chat_completions])
        .setup(|app| {
            if let Some(dir) = app.path().app_data_dir().ok() {
                let _ = std::fs::create_dir_all(dir);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running AI Assistant Client");
}
