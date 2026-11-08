//! Files in `%LOCALAPPDATA%\PCCleaner`: settings, window placement, cleanup history, logs and the
//! `.reg`/`.xml` backups. The names and formats are the ones the PowerShell version used, so an
//! update keeps everything.

pub mod config;
pub mod history;
pub mod json;
pub mod logger;
pub mod paths;
pub mod window_prefs;
