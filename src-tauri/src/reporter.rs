use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use pccleaner_core::report::{Reporter, StepState, Tone};
use pccleaner_core::storage::logger::Logger;
use pccleaner_core::Text;

/// Forwards core progress to the window as events and to the log file.
pub struct EventReporter {
    pub app: AppHandle,
    pub logger: Arc<Logger>,
}

#[derive(Serialize, Clone)]
struct Line {
    text: Text,
    tone: Tone,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Step<'a> {
    id: &'a str,
    state: StepState,
    bytes: f64,
    note: Option<Text>,
}

impl Reporter for EventReporter {
    fn status(&self, text: Text) {
        let _ = self.app.emit("pcc://status", text);
    }

    fn line(&self, text: Text, tone: Tone) {
        self.logger.write(&text.english());
        let _ = self.app.emit("pcc://line", Line { text, tone });
    }

    fn step(&self, id: &str, state: StepState, bytes: f64, note: Option<Text>) {
        let _ = self.app.emit("pcc://step", Step { id, state, bytes, note });
    }

    fn log(&self, line: &str) {
        self.logger.write(line);
    }
}
