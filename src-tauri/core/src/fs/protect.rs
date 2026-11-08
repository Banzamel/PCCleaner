use std::path::Path;

use crate::user::UserContext;

/// Folders in a drive root that belong to Windows or hardware vendors.
const ROOT_SYSTEM_FOLDERS: &[&str] = &[
    "$recycle.bin", "system volume information", "recovery", "config.msi", "msocache", "perflogs", "boot", "windows",
    "windows.old", "program files", "program files (x86)", "programdata", "users", "documents and settings",
    "onedrivetemp", "intel", "amd", "nvidia", "drivers", "efi",
];

/// Folders a leftover rule may never offer and the cleaner may never delete, whatever a rule found:
/// a drive root, system folders in a root, Windows, Program Files, ProgramData, the profile and its
/// AppData roots, the Users folder and PC Cleaner's own folder - and any parent of those.
#[derive(Debug, Clone)]
pub struct Protected {
    roots: Vec<String>,
}

fn norm(p: &str) -> String {
    p.trim().replace('/', "\\").trim_end_matches('\\').to_lowercase()
}

impl Protected {
    pub fn new(user: &UserContext, app_dir: &Path) -> Self {
        let users = user.profile.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let list = [
            &user.windir, &user.program_files, &user.program_files_x86, &user.program_data, &user.profile, &user.appdata,
            &user.local_appdata, &user.local_low(), &users, &app_dir.to_path_buf(),
        ];
        Self::from_paths(list.iter().map(|p| p.to_string_lossy().to_string()))
    }

    pub fn from_paths<I: IntoIterator<Item = String>>(paths: I) -> Self {
        Self { roots: paths.into_iter().map(|p| norm(&p)).filter(|p| !p.is_empty()).collect() }
    }

    pub fn is_protected(&self, path: &str) -> bool {
        let s = norm(path);
        let b = s.as_bytes();
        // Only "X:\something" - a drive root or an odd path is always protected
        if b.len() < 4 || !b[0].is_ascii_alphabetic() || b[1] != b':' || b[2] != b'\\' {
            return true;
        }
        let parts: Vec<&str> = s.split('\\').collect();
        if parts.len() == 2 && (ROOT_SYSTEM_FOLDERS.contains(&parts[1]) || parts[1].starts_with('$')) {
            return true;
        }
        let with_sep = format!("{s}\\");
        // The protected folder itself or a parent of it (e.g. P:\PCCleaner when the program is in P:\PCCleaner\app)
        self.roots.iter().any(|r| *r == s || r.starts_with(&with_sep))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> Protected {
        Protected::from_paths(["C:\\Windows", "C:\\Users\\Jan", "C:\\Users", "P:\\Tools\\PCCleaner\\app"].map(String::from))
    }

    #[test]
    fn roots_and_system_folders() {
        let p = p();
        assert!(p.is_protected("G:\\"));
        assert!(p.is_protected("G:"));
        assert!(p.is_protected("G:\\$RECYCLE.BIN"));
        assert!(p.is_protected("D:\\Program Files"));
        assert!(!p.is_protected("G:\\Steam\\steamapps\\common\\Old Game"));
    }

    #[test]
    fn protected_folders_and_their_parents() {
        let p = p();
        assert!(p.is_protected("c:\\windows\\"));
        assert!(p.is_protected("C:\\Users\\Jan"));
        assert!(p.is_protected("P:\\Tools"));
        assert!(!p.is_protected("C:\\Users\\Jan\\AppData\\Roaming\\OldApp"));
    }
}
