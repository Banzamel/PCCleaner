use serde::Serialize;
use std::time::SystemTime;

use crate::text::Text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LeftoverKind {
    Folder,
    File,
    Registry,
    Regvalue,
    Task,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

/// A leftover of an uninstalled program or game, or a large file the user may want to remove.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Leftover {
    /// R1, R2... in scan order
    pub id: String,
    pub kind: LeftoverKind,
    /// Folder or file path, registry key path, or task path.
    pub path: String,
    pub name: String,
    pub bytes: f64,
    /// yyyy-MM-dd
    pub last_write: Option<String>,
    pub confidence: Confidence,
    pub reason: Text,
    pub game: bool,
    pub large_file: bool,
    /// First registry subkeys (often game names) - the AI describes vendor keys from them.
    pub subkeys: Vec<String>,
    /// Registry value name of a dead startup entry.
    pub value_name: Option<String>,
    /// Task Scheduler folder of a task.
    pub folder: Option<String>,
}

impl Leftover {
    pub fn new(kind: LeftoverKind, path: impl Into<String>, name: impl Into<String>, confidence: Confidence, reason: Text) -> Self {
        Self {
            id: String::new(),
            kind,
            path: path.into(),
            name: name.into(),
            bytes: 0.0,
            last_write: None,
            confidence,
            reason,
            game: false,
            large_file: false,
            subkeys: Vec::new(),
            value_name: None,
            folder: None,
        }
    }

    /// Folders and files go to the Recycle Bin; registry entries and tasks get a backup instead.
    pub fn goes_to_recycle_bin(&self) -> bool {
        matches!(self.kind, LeftoverKind::Folder | LeftoverKind::File)
    }

    pub fn with_date(mut self, t: Option<SystemTime>) -> Self {
        self.last_write = t.map(date);
        self
    }
}

/// Local date as yyyy-MM-dd.
pub fn date(t: SystemTime) -> String {
    chrono::DateTime::<chrono::Local>::from(t).format("%Y-%m-%d").to_string()
}

/// Days since a time (0 for the future or unknown).
pub fn days_since(t: Option<SystemTime>) -> f64 {
    t.and_then(|t| SystemTime::now().duration_since(t).ok()).map(|d| d.as_secs_f64() / 86_400.0).unwrap_or(0.0)
}
