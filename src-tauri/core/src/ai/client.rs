use std::io::{BufRead, BufReader};
use std::time::Duration;

use serde_json::{json, Value};

use super::base_url;

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(timeout).build()
}

/// Whether the private server answers.
pub fn ping() -> bool {
    agent(Duration::from_secs(2)).get(&format!("{}/api/version", base_url())).call().is_ok()
}

/// Whether the model is downloaded (also as `name:latest`).
pub fn has_model(model: &str) -> bool {
    let Ok(r) = agent(Duration::from_secs(5)).get(&format!("{}/api/tags", base_url())).call() else { return false };
    let Ok(v) = r.into_json::<Value>() else { return false };
    let latest = format!("{model}:latest");
    v["models"].as_array().is_some_and(|a| a.iter().any(|m| m["name"] == model || m["name"] == latest.as_str()))
}

/// Ollama describes errors in the body (`{"error": "..."}`); without it only the status remains.
fn error_text(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, r) => {
            let body = r.into_string().unwrap_or_default();
            format!("HTTP {code} {body}").trim().to_string()
        }
        other => other.to_string(),
    }
}

/// One chat request without streaming. `format` is a JSON schema for structured answers.
pub fn chat(model: &str, messages: &Value, format: Option<&Value>) -> Result<String, String> {
    let mut body = json!({
        "model": model, "messages": messages, "stream": false, "keep_alive": "5m",
        "options": { "temperature": 0.2, "num_ctx": 12288 },
    });
    if let Some(f) = format {
        body["format"] = f.clone();
    }
    let r = agent(Duration::from_secs(600)).post(&format!("{}/api/chat", base_url())).send_json(body).map_err(error_text)?;
    let v: Value = r.into_json().map_err(|e| e.to_string())?;
    let content = v["message"]["content"].as_str().unwrap_or_default().to_string();
    if content.is_empty() {
        if let Some(reason) = v["done_reason"].as_str().filter(|r| *r != "stop") {
            return Err(format!("Ollama: done_reason={reason}"));
        }
    }
    Ok(content)
}

/// Downloads a model through the server, reporting (completed, total) bytes.
pub fn pull(model: &str, progress: &dyn Fn(u64, u64)) -> Result<(), String> {
    let r = agent(Duration::from_secs(24 * 3600))
        .post(&format!("{}/api/pull", base_url()))
        .send_json(json!({ "model": model, "stream": true }))
        .map_err(error_text)?;
    for line in BufReader::new(r.into_reader()).lines() {
        let line = line.map_err(|e| e.to_string())?;
        let Ok(v) = serde_json::from_str::<Value>(&line) else { continue };
        if let Some(err) = v["error"].as_str() {
            return Err(err.to_string());
        }
        if let (Some(c), Some(t)) = (v["completed"].as_u64(), v["total"].as_u64()) {
            progress(c, t);
        }
        if v["status"] == "success" {
            return Ok(());
        }
    }
    Err("the download ended early".into())
}
