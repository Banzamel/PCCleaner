use std::path::{Path, PathBuf};

use crate::system::registry;

/// The account whose profile is cleaned. On a standard account UAC asks for ANOTHER administrator's
/// password and the program then runs as that account; the elevation passes the SID of the person
/// who started it, so their folders, registry and Recycle Bin are cleaned, not the administrator's.
#[derive(Debug, Clone)]
pub struct UserContext {
    pub sid: String,
    pub other_user: bool,
    pub profile: PathBuf,
    pub appdata: PathBuf,
    pub local_appdata: PathBuf,
    pub temp: PathBuf,
    /// `HKEY_CURRENT_USER` or `HKEY_USERS\<SID>` for another account.
    pub hkcu: String,
    pub windir: PathBuf,
    /// Usually `C:`, but Windows may live on another drive.
    pub system_drive: String,
    pub program_data: PathBuf,
    pub program_files: PathBuf,
    pub program_files_x86: PathBuf,
    pub public: PathBuf,
}

fn env_path(name: &str, fallback: &str) -> PathBuf {
    PathBuf::from(std::env::var(name).unwrap_or_else(|_| fallback.to_string()))
}

impl UserContext {
    /// Current process account, or the account given by `--sid` when it has a loaded profile.
    pub fn detect(requested_sid: Option<&str>) -> Self {
        let my_sid = crate::system::elevation::current_sid().unwrap_or_default();
        let mut c = Self {
            sid: my_sid.clone(),
            other_user: false,
            profile: env_path("USERPROFILE", "C:\\Users\\Default"),
            appdata: env_path("APPDATA", "C:\\Users\\Default\\AppData\\Roaming"),
            local_appdata: env_path("LOCALAPPDATA", "C:\\Users\\Default\\AppData\\Local"),
            temp: env_path("TEMP", "C:\\Windows\\Temp"),
            hkcu: "HKEY_CURRENT_USER".into(),
            windir: env_path("windir", "C:\\Windows"),
            system_drive: std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into()),
            program_data: env_path("ProgramData", "C:\\ProgramData"),
            program_files: env_path("ProgramFiles", "C:\\Program Files"),
            program_files_x86: env_path("ProgramFiles(x86)", "C:\\Program Files (x86)"),
            public: env_path("PUBLIC", "C:\\Users\\Public"),
        };
        if let Some(sid) = requested_sid.filter(|s| !s.is_empty() && *s != my_sid) {
            c.switch_to(sid);
        }
        c
    }

    fn switch_to(&mut self, sid: &str) {
        let key = format!("HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList\\{sid}");
        let Some(profile) = registry::read_string(&key, "ProfileImagePath") else { return };
        let profile = PathBuf::from(registry::expand_env(&profile));
        if !profile.is_dir() || !registry::key_exists(&format!("HKEY_USERS\\{sid}")) {
            return;
        }
        self.sid = sid.to_string();
        self.other_user = true;
        self.hkcu = format!("HKEY_USERS\\{sid}");
        self.profile = profile.clone();
        self.appdata = self.shell_folder("AppData", &profile.join("AppData\\Roaming"));
        self.local_appdata = self.shell_folder("Local AppData", &profile.join("AppData\\Local"));
        self.temp = self.local_appdata.join("Temp");
    }

    /// A user shell folder, which may be relocated (Documents in OneDrive, Downloads on D:).
    pub fn shell_folder(&self, name: &str, fallback: &Path) -> PathBuf {
        let key = format!("{}\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders", self.hkcu);
        if let Some(v) = registry::read_string(&key, name) {
            let profile = self.profile.to_string_lossy().to_string();
            let p = registry::expand_env(&v.replace("%USERPROFILE%", &profile));
            let p = PathBuf::from(p.trim_end_matches('\\'));
            if p.is_dir() {
                return p;
            }
        }
        fallback.to_path_buf()
    }

    pub fn downloads(&self) -> PathBuf {
        self.shell_folder("{374DE290-123F-4565-9164-39C4925E467B}", &self.profile.join("Downloads"))
    }

    pub fn desktop(&self) -> PathBuf {
        self.shell_folder("Desktop", &self.profile.join("Desktop"))
    }

    pub fn documents(&self) -> PathBuf {
        self.shell_folder("Personal", &self.profile.join("Documents"))
    }

    pub fn saved_games(&self) -> PathBuf {
        self.shell_folder("{4C5C32FF-BB9D-43b0-B5B4-2D72E54EAAA4}", &self.profile.join("Saved Games"))
    }

    pub fn local_low(&self) -> PathBuf {
        self.profile.join("AppData\\LocalLow")
    }

    /// Path in the root of the system drive, e.g. `sys_path("NVIDIA")` -> `C:\NVIDIA`.
    pub fn sys_path(&self, rel: &str) -> PathBuf {
        PathBuf::from(format!("{}\\{}", self.system_drive, rel))
    }

    /// A context with fixed paths under `root`, for tests.
    pub fn for_tests(root: &Path) -> Self {
        Self {
            sid: "S-1-5-21-test".into(),
            other_user: false,
            profile: root.join("profile"),
            appdata: root.join("profile/AppData/Roaming"),
            local_appdata: root.join("profile/AppData/Local"),
            temp: root.join("profile/AppData/Local/Temp"),
            hkcu: "HKEY_CURRENT_USER".into(),
            windir: root.join("Windows"),
            system_drive: "C:".into(),
            program_data: root.join("ProgramData"),
            program_files: root.join("Program Files"),
            program_files_x86: root.join("Program Files (x86)"),
            public: root.join("Users/Public"),
        }
    }
}
