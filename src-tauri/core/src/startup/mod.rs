//! Startup apps: programs that start when the user signs in. Nothing is deleted when a program is
//! turned off: Run and Startup-folder entries are disabled the way Task Manager does it (StartupApproved),
//! tasks through Task Scheduler. Turning one back on restores exactly the previous state.

pub mod actions;
pub mod approved;
pub mod entries;
pub mod protection;
pub mod sources;

use serde::Serialize;

use crate::text::Text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Run,
    Folder,
    Task,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupEntry {
    /// S1, S2...
    pub id: String,
    pub kind: EntryKind,
    pub name: String,
    pub command: String,
    pub file: Option<String>,
    pub source: Text,
    pub exists: bool,
    pub dead: bool,
    pub publisher: String,
    /// Why it is read-only (security software, drivers, updaters, parts of Windows).
    pub protected: Option<Text>,
    pub is_enabled: bool,
    #[serde(skip)]
    pub reg_key: String,
    #[serde(skip)]
    pub approved_key: String,
    #[serde(skip)]
    pub approved_name: String,
    /// Full path of a task, or of a Startup-folder file.
    #[serde(skip)]
    pub path: String,
}
