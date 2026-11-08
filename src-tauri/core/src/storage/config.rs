use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::json::{as_bool, as_f64, as_string, as_strings, read_object, write};
use super::paths::DataPaths;

/// The Settings page (settings.json). Command-line parameters win over it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// auto, pl, en or es
    pub language: String,
    /// auto, light or dark ("" = not set, window.json may hold an older choice)
    pub theme: String,
    /// "" = the model chosen in the installer
    pub model: String,
    pub assistant: bool,
    pub check_updates: bool,
    pub preview: bool,
    pub skip_categories: Vec<String>,
    pub scan_leftovers: bool,
    pub scan_large_files: bool,
    pub excluded_folders: Vec<String>,
    /// Run logs older than that are deleted at start; 0 keeps all.
    pub log_days: u32,
    pub close_to_tray: bool,
    /// off, logon, daily or weekly
    pub auto_clean: String,
    /// yyyy-MM-dd of the last automatic cleaning
    pub auto_clean_last: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: "auto".into(),
            theme: String::new(),
            model: String::new(),
            assistant: true,
            check_updates: true,
            preview: false,
            skip_categories: Vec::new(),
            scan_leftovers: true,
            scan_large_files: true,
            excluded_folders: Vec::new(),
            log_days: 30,
            close_to_tray: false,
            auto_clean: "off".into(),
            auto_clean_last: String::new(),
        }
    }
}

/// Polish keys and values written before 1.6.4 (ustawienia.json).
const OLD_KEYS: &[(&str, &str)] = &[("Jezyk", "Language"), ("Motyw", "Theme"), ("Asystent", "Assistant"), ("Aktualizacje", "CheckUpdates"), ("Podglad", "Preview")];
pub const OLD_VALUES: &[(&str, &str)] = &[
    ("jasny", "light"), ("ciemny", "dark"), ("przeglad", "overview"), ("analiza", "analysis"), ("autostart", "startup"),
    ("sterowniki", "drivers"), ("czat", "chat"), ("ustawienia", "settings"), ("czyszczenie", "cleanup"), ("dziennik", "log"),
];

pub fn renamed(map: Map<String, Value>, keys: &[(&str, &str)]) -> Map<String, Value> {
    map.into_iter()
        .map(|(k, v)| {
            let key = keys.iter().find(|(o, _)| *o == k).map(|(_, n)| n.to_string()).unwrap_or(k);
            let value = match &v {
                Value::String(s) => OLD_VALUES.iter().find(|(o, _)| o == s).map(|(_, n)| json!(n)).unwrap_or(v),
                _ => v,
            };
            (key, value)
        })
        .collect()
}

impl Config {
    pub fn load(paths: &DataPaths) -> Self {
        let map = read_object(&paths.settings()).or_else(|| read_object(&paths.legacy_settings()).map(|m| renamed(m, OLD_KEYS)));
        map.map(|m| Self::from_map(&m)).unwrap_or_default()
    }

    pub fn from_map(m: &Map<String, Value>) -> Self {
        let mut c = Self::default();
        let s = |k: &str| m.get(k).and_then(as_string);
        let b = |k: &str| m.get(k).and_then(as_bool);
        if let Some(v) = s("Language").filter(|v| matches!(v.as_str(), "auto" | "pl" | "en" | "es")) { c.language = v; }
        if let Some(v) = s("Theme").filter(|v| matches!(v.as_str(), "" | "auto" | "light" | "dark")) { c.theme = v; }
        if let Some(v) = s("Model") { c.model = v; }
        if let Some(v) = b("Assistant") { c.assistant = v; }
        if let Some(v) = b("CheckUpdates") { c.check_updates = v; }
        if let Some(v) = b("Preview") { c.preview = v; }
        if let Some(v) = m.get("SkipCategories") { c.skip_categories = as_strings(v); }
        if let Some(v) = b("ScanLeftovers") { c.scan_leftovers = v; }
        if let Some(v) = b("ScanLargeFiles") { c.scan_large_files = v; }
        if let Some(v) = m.get("ExcludedFolders") { c.excluded_folders = as_strings(v); }
        if let Some(v) = m.get("LogDays").and_then(as_f64) { c.log_days = v.max(0.0) as u32; }
        if let Some(v) = b("CloseToTray") { c.close_to_tray = v; }
        if let Some(v) = s("AutoClean").filter(|v| matches!(v.as_str(), "off" | "logon" | "daily" | "weekly")) { c.auto_clean = v; }
        if let Some(v) = s("AutoCleanLast") { c.auto_clean_last = v; }
        c
    }

    /// The PascalCase file format the PowerShell version wrote and read.
    pub fn to_value(&self) -> Value {
        json!({
            "Language": self.language, "Theme": self.theme, "Model": self.model, "Assistant": self.assistant,
            "CheckUpdates": self.check_updates, "Preview": self.preview, "SkipCategories": self.skip_categories,
            "ScanLeftovers": self.scan_leftovers, "ScanLargeFiles": self.scan_large_files, "ExcludedFolders": self.excluded_folders,
            "LogDays": self.log_days, "CloseToTray": self.close_to_tray, "AutoClean": self.auto_clean, "AutoCleanLast": self.auto_clean_last,
        })
    }

    pub fn save(&self, paths: &DataPaths) -> std::io::Result<()> {
        write(&paths.settings(), &self.to_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_powershell_file_with_single_values() {
        let m: Map<String, Value> = serde_json::from_str(r#"{"Language":"es","SkipCategories":"dns","LogDays":"7","Assistant":false,"Bogus":1}"#).unwrap();
        let c = Config::from_map(&m);
        assert_eq!(c.language, "es");
        assert_eq!(c.skip_categories, vec!["dns"]);
        assert_eq!(c.log_days, 7);
        assert!(!c.assistant);
    }

    #[test]
    fn legacy_polish_keys_and_values() {
        let m: Map<String, Value> = serde_json::from_str(r#"{"Jezyk":"pl","Motyw":"ciemny","Podglad":true}"#).unwrap();
        let c = Config::from_map(&renamed(m, OLD_KEYS));
        assert_eq!((c.language.as_str(), c.theme.as_str(), c.preview), ("pl", "dark", true));
    }

    #[test]
    fn round_trip() {
        let c = Config { auto_clean: "weekly".into(), excluded_folders: vec!["D:\\Keep".into()], ..Default::default() };
        let back = Config::from_map(c.to_value().as_object().unwrap());
        assert_eq!(back, c);
    }
}
