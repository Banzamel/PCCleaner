use serde::Serialize;

use crate::text::Text;

/// The one error type that crosses the IPC boundary. `message` is a translatable text, so the UI
/// shows it in the user's language; `detail` keeps the raw OS/library message for the log.
#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[error("{detail}")]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub message: Text,
    pub detail: String,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(message: Text, detail: impl Into<String>) -> Self {
        Self { message, detail: detail.into() }
    }

    /// An error whose only description is the OS or library message.
    pub fn raw(detail: impl Into<String>) -> Self {
        let detail = detail.into();
        Self { message: Text::new("An error occurred: {0}").arg(&detail), detail }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::raw(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::raw(e.to_string())
    }
}
