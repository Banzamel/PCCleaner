use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use super::{blocking, lock};
use pccleaner_core::ai::{client, install, models, server};
use pccleaner_core::{AppError, AppResult, Text};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub installed: bool,
    pub models: Vec<String>,
    /// The model chosen when the AI was installed (config.json) or the default.
    pub installer_model: String,
}

#[derive(Serialize, Clone)]
struct Progress {
    text: Text,
    fraction: f64,
}

#[tauri::command]
pub async fn ai_status(app: AppHandle) -> AppResult<AiStatus> {
    blocking(app, |_, st| {
        let ai = st.ai_dir();
        Ok(AiStatus { installed: server::exe(&ai).is_file(), models: models::local_models(&ai), installer_model: models::installer_model(&st.app_dir) })
    })
    .await
}

/// Starts the private server on first use and checks the model (the first use takes 10-20 s).
#[tauri::command]
pub async fn ai_prepare(app: AppHandle, model: String) -> AppResult<()> {
    blocking(app, move |_, st| {
        let ai = st.ai_dir();
        let fail = |t: Text| {
            st.log(&format!("Model: {}", t.english()));
            Err(AppError::new(t, "assistant unavailable"))
        };
        if !server::exe(&ai).is_file() {
            return fail(Text::new("The AI assistant is not installed. Install it in Settings > AI assistant."));
        }
        if !lock(&st.ollama)?.start(&ai) {
            return fail(Text::new("Could not start Ollama from the ai\\ folder - the assistant is unavailable."));
        }
        if !client::has_model(&model) {
            return fail(Text::new("Model {0} is missing in ai\\models (Settings > AI assistant > Install) - the assistant is unavailable.").arg(&model));
        }
        Ok(())
    })
    .await
}

/// One request to the local model; `format` is a JSON schema for structured answers.
#[tauri::command]
pub async fn ai_chat(app: AppHandle, model: String, messages: Value, format: Option<Value>) -> AppResult<String> {
    blocking(app, move |_, st| {
        client::chat(&model, &messages, format.as_ref()).map_err(|e| {
            st.log(&format!("Model error: {e}"));
            AppError::new(Text::new("Model error: {0}").arg(&e), e)
        })
    })
    .await
}

#[tauri::command]
pub async fn ai_stop(app: AppHandle) -> AppResult<()> {
    blocking(app, |_, st| {
        lock(&st.ollama)?.stop();
        Ok(())
    })
    .await
}

/// Downloads the AI engine and the model into the program folder (Settings). Progress goes to the
/// window as `pcc://ai-install`.
#[tauri::command]
pub async fn ai_install(app: AppHandle, model: String) -> AppResult<()> {
    blocking(app, move |app, st| {
        lock(&st.ollama)?.stop();
        st.log(&format!("AI install: {model}"));
        let emit = |text: Text, fraction: f64| {
            let _ = app.emit("pcc://ai-install", Progress { text, fraction });
        };
        install::install(&st.app_dir, &model, &emit).map_err(|e| {
            st.log(&format!("AI install failed: {e}"));
            AppError::new(Text::new("AI setup failed: {0}").arg(&e), e)
        })
    })
    .await
}
