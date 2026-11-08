//! Newer drivers from Windows Update - the same as Settings > Windows Update > Optional updates.
//! Windows Update is contacted only when the user clicks "Check", never at start.

pub mod devices;
pub mod wu;

use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

use crate::text::Text;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverUpdate {
    /// Windows Update id
    pub id: String,
    pub title: String,
    /// Device name (or the model when no device matches)
    pub name: String,
    pub model: String,
    pub provider: String,
    pub class: String,
    pub bytes: f64,
    pub version: String,
    pub new_date: Option<String>,
    pub current_version: Option<String>,
    pub current_date: Option<String>,
    pub problem: Option<Text>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDevice {
    pub name: String,
    pub id: String,
    pub code: u32,
    pub description: Text,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverCheck {
    pub drivers: Vec<DriverUpdate>,
    pub problems: Vec<ProblemDevice>,
}

static VERSION_AT_END: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+(\.\d+){1,3})\s*$").expect("valid regex"));

/// Version from the update title, e.g. "Intel Corporation - Display - 31.0.101.4502".
pub fn version_from_title(t: &str) -> String {
    VERSION_AT_END.captures(t).map(|c| c[1].to_string()).unwrap_or_default()
}

/// Device Manager problem code in words.
pub fn problem_description(code: u32) -> Text {
    match code {
        28 => Text::new("no driver installed"),
        10 => Text::new("the device cannot start (code 10)"),
        43 => Text::new("the device reported a problem (code 43)"),
        1 | 18 | 31 | 39 | 52 => Text::new("the driver does not work properly (code {0})").arg(code),
        _ => Text::new("device problem (code {0})").arg(code),
    }
}

/// OLE automation DATE (days since 1899-12-30) as yyyy-MM-dd; nothing for dates before 1990.
pub fn ole_date(days: f64) -> Option<String> {
    let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30)?;
    let d = base.checked_add_signed(chrono::Duration::days(days.floor() as i64))?;
    (d > chrono::NaiveDate::from_ymd_opt(1990, 1, 1)?).then(|| d.format("%Y-%m-%d").to_string())
}

/// WMI datetime "20230115000000.000000-000" as yyyy-MM-dd.
pub fn wmi_date(s: &str) -> Option<String> {
    if s.len() < 8 || !s.is_char_boundary(8) || s.starts_with("1") && &s[..4] < "1990" {
        return None;
    }
    Some(format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_and_dates() {
        assert_eq!(version_from_title("Intel Corporation - Display - 31.0.101.4502"), "31.0.101.4502");
        assert_eq!(version_from_title("No version"), "");
        assert_eq!(ole_date(45000.5).as_deref(), Some("2023-03-15"));
        assert_eq!(ole_date(0.0), None);
        assert_eq!(wmi_date("20230115000000.000000-000").as_deref(), Some("2023-01-15"));
        assert_eq!(problem_description(31).english(), "the driver does not work properly (code 31)");
    }
}
