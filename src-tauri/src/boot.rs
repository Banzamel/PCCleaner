use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use pccleaner_core::autoclean;
use pccleaner_core::cli::{self, CliArgs};
use pccleaner_core::storage::config::Config;
use pccleaner_core::storage::logger::{self, Logger};
use pccleaner_core::storage::paths::DataPaths;
use pccleaner_core::storage::window_prefs::WindowPrefs;
use pccleaner_core::system::{elevation, locale};
use pccleaner_core::user::UserContext;

use crate::state::AppState;

pub const APP_ID: &str = "pl.banzamel.pccleaner";

/// Prepares the program before any window exists. Returns `None` when this process should end:
/// it relaunched itself through UAC, the user declined UAC, or an automatic cleaning is not due.
pub fn prepare() -> Option<AppState> {
    let args: CliArgs = cli::parse(std::env::args().skip(1));
    // Administrator rights for every mode, like the old launcher: relaunch through UAC with the
    // starting account's SID, so another administrator's password still cleans this user's profile.
    if !elevation::is_admin() {
        let sid = elevation::current_sid().unwrap_or_default();
        elevation::relaunch_elevated(&cli::to_args(&args, &sid));
        return None;
    }
    locale::set_app_user_model_id(APP_ID);
    let user = UserContext::detect(args.sid.as_deref());
    let data = DataPaths::new(&user.local_appdata);
    let config = Config::load(&data);
    if args.auto_clean && !autoclean::is_due(&config.auto_clean, &config.auto_clean_last, chrono::Local::now().date_naive()) {
        return None;
    }
    let exe = std::env::current_exe().unwrap_or_default();
    let app_dir = exe.parent().map(PathBuf::from).unwrap_or_default();
    let log = Arc::new(Logger::new(&data.logs()));
    if config.log_days > 0 {
        let n = logger::remove_old(&data.logs(), &log.path, config.log_days);
        if n > 0 {
            log.write(&format!("Deleted {n} logs older than {} days.", config.log_days));
        }
    }
    // The old uninstaller removed the task; a moved program needs the new path - keep it in sync
    if !user.other_user && config.auto_clean != "off" {
        if let Err(e) = autoclean::apply(&config.auto_clean, &exe) {
            log.write(&format!("Automatic cleaning task: {e}"));
        }
    }
    let window = WindowPrefs::load(&data).unwrap_or_default();
    Some(AppState {
        cli: args,
        user,
        data,
        app_dir,
        logger: log,
        config: Mutex::new(config),
        window: Mutex::new(window),
        session: Mutex::new(None),
        startup: Mutex::new(Vec::new()),
        ollama: Mutex::new(Default::default()),
        cleaning: AtomicBool::new(false),
        installing_drivers: AtomicBool::new(false),
        exiting: AtomicBool::new(false),
    })
}
