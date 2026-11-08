use std::path::PathBuf;

use crate::text::Text;
use crate::user::UserContext;

const BASE: &str = "Software\\Microsoft\\Windows\\CurrentVersion";

/// A Run key with its StartupApproved counterpart (where Windows and Task Manager keep the on/off state).
pub struct RunKey {
    pub reg_key: String,
    pub approved_key: String,
    pub source: Text,
}

/// A Startup folder with its StartupApproved key.
pub struct StartupFolder {
    pub folder: PathBuf,
    pub approved_key: String,
    pub source: Text,
}

pub fn run_keys(u: &UserContext) -> Vec<RunKey> {
    let hkcu = &u.hkcu;
    vec![
        RunKey { reg_key: format!("{hkcu}\\{BASE}\\Run"), approved_key: format!("{hkcu}\\{BASE}\\Explorer\\StartupApproved\\Run"), source: Text::new("Registry (user)") },
        RunKey {
            reg_key: format!("HKEY_LOCAL_MACHINE\\{BASE}\\Run"),
            approved_key: format!("HKEY_LOCAL_MACHINE\\{BASE}\\Explorer\\StartupApproved\\Run"),
            source: Text::new("Registry (all users)"),
        },
        RunKey {
            reg_key: "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run".into(),
            approved_key: format!("HKEY_LOCAL_MACHINE\\{BASE}\\Explorer\\StartupApproved\\Run32"),
            source: Text::new("Registry (all users, 32-bit)"),
        },
    ]
}

pub fn startup_folders(u: &UserContext) -> Vec<StartupFolder> {
    vec![
        StartupFolder {
            folder: u.appdata.join("Microsoft\\Windows\\Start Menu\\Programs\\Startup"),
            approved_key: format!("{}\\{BASE}\\Explorer\\StartupApproved\\StartupFolder", u.hkcu),
            source: Text::new("Startup folder (user)"),
        },
        StartupFolder {
            folder: u.program_data.join("Microsoft\\Windows\\Start Menu\\Programs\\StartUp"),
            approved_key: format!("HKEY_LOCAL_MACHINE\\{BASE}\\Explorer\\StartupApproved\\StartupFolder"),
            source: Text::new("Startup folder (all users)"),
        },
    ]
}
