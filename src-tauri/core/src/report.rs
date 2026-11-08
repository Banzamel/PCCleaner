use serde::Serialize;

use crate::text::Text;

/// Colors of the Log page lines - the console colors the old program used.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Tone {
    Gray,
    DarkGray,
    White,
    Green,
    Yellow,
    Cyan,
}

/// States of a cleanup step on the progress screen.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StepState {
    Running,
    Ok,
    Partial,
    Skipped,
    Fail,
}

/// Where the core reports progress. The app forwards it to the window as events and to the log file;
/// tests use [`NullReporter`].
pub trait Reporter: Send + Sync {
    /// Progress line that replaces the previous one (status bar).
    fn status(&self, text: Text);
    /// Line on the Log page (also written to the log file in English).
    fn line(&self, text: Text, tone: Tone);
    /// Cleanup step update.
    fn step(&self, id: &str, state: StepState, bytes: f64, note: Option<Text>);
    /// Log file only (English, technical).
    fn log(&self, line: &str);
}

pub struct NullReporter;

impl Reporter for NullReporter {
    fn status(&self, _: Text) {}
    fn line(&self, _: Text, _: Tone) {}
    fn step(&self, _: &str, _: StepState, _: f64, _: Option<Text>) {}
    fn log(&self, _: &str) {}
}
