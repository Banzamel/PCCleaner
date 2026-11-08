use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use pccleaner_core::ai::server::OllamaServer;
use pccleaner_core::cli::CliArgs;
use pccleaner_core::scan::session::ScanSession;
use pccleaner_core::startup::StartupEntry;
use pccleaner_core::storage::config::Config;
use pccleaner_core::storage::logger::Logger;
use pccleaner_core::storage::paths::DataPaths;
use pccleaner_core::storage::window_prefs::WindowPrefs;
use pccleaner_core::user::UserContext;

/// Everything the commands share. One instance, managed by Tauri.
pub struct AppState {
    pub cli: CliArgs,
    pub user: UserContext,
    pub data: DataPaths,
    /// Folder of PCCleaner.exe (the AI engine lives in `ai\` next to it).
    pub app_dir: PathBuf,
    pub logger: Arc<Logger>,
    pub config: Mutex<Config>,
    pub window: Mutex<WindowPrefs>,
    pub session: Mutex<Option<ScanSession>>,
    pub startup: Mutex<Vec<StartupEntry>>,
    pub ollama: Mutex<OllamaServer>,
    /// The window must not close during these (half-done cleaning, a driver install).
    pub cleaning: AtomicBool,
    pub installing_drivers: AtomicBool,
    /// Set by Exit in the tray menu, an update or a restart: the X-to-tray rule no longer applies.
    pub exiting: AtomicBool,
}

impl AppState {
    pub fn ai_dir(&self) -> PathBuf {
        self.app_dir.join("ai")
    }

    pub fn log(&self, line: &str) {
        self.logger.write(line);
    }
}
