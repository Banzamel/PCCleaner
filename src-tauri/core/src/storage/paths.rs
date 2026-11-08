use std::path::{Path, PathBuf};

/// Where PC Cleaner keeps its data for one account.
#[derive(Debug, Clone)]
pub struct DataPaths {
    pub root: PathBuf,
}

impl DataPaths {
    pub fn new(local_appdata: &Path) -> Self {
        Self { root: local_appdata.join("PCCleaner") }
    }

    pub fn settings(&self) -> PathBuf {
        self.root.join("settings.json")
    }

    /// Settings saved by versions before 1.6.4 (Polish keys).
    pub fn legacy_settings(&self) -> PathBuf {
        self.root.join("ustawienia.json")
    }

    pub fn window(&self) -> PathBuf {
        self.root.join("window.json")
    }

    pub fn legacy_window(&self) -> PathBuf {
        self.root.join("okno.json")
    }

    pub fn history(&self) -> PathBuf {
        self.root.join("history.json")
    }

    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// `.reg` and `.xml` backups of one run; the folder name is kept from the first versions.
    pub fn backups(&self, stamp: &str) -> PathBuf {
        self.logs().join("kopie_rejestru").join(stamp)
    }
}

/// `yyyy-MM-dd_HH-mm` of the local time, used in log and backup names.
pub fn stamp() -> String {
    chrono::Local::now().format("%Y-%m-%d_%H-%M").to_string()
}

/// A file name made safe from an item name (registry key, task, startup entry).
pub fn safe_name(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | ' ') { c } else { '_' }).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn safe_names() {
        assert_eq!(super::safe_name("Klei\\Don't Starve"), "Klei_Don_t Starve");
    }
}
