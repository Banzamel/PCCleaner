use serde::Serialize;
use serde_json::{json, Map, Value};

use super::config::renamed;
use super::json::{as_bool, as_f64, as_string, read_object, write};
use super::paths::DataPaths;

/// window.json: size and position (physical pixels), maximised, the display scale it was saved at,
/// the last page and an older theme field.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPrefs {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub max: bool,
    pub scale: f64,
    pub page: String,
    pub theme: String,
}

const OLD_KEYS: &[(&str, &str)] = &[("Skala", "Scale"), ("Strona", "Page"), ("Motyw", "Theme")];

impl WindowPrefs {
    pub fn load(paths: &DataPaths) -> Option<Self> {
        let m = read_object(&paths.window()).or_else(|| read_object(&paths.legacy_window()).map(|m| renamed(m, OLD_KEYS)))?;
        Some(Self::from_map(&m))
    }

    pub fn from_map(m: &Map<String, Value>) -> Self {
        let n = |k: &str| m.get(k).and_then(as_f64).unwrap_or(0.0);
        Self {
            x: n("X") as i32,
            y: n("Y") as i32,
            w: n("W") as i32,
            h: n("H") as i32,
            max: m.get("Max").and_then(as_bool).unwrap_or(false),
            scale: n("Scale"),
            page: m.get("Page").and_then(as_string).unwrap_or_default(),
            theme: m.get("Theme").and_then(as_string).unwrap_or_default(),
        }
    }

    pub fn save(&self, paths: &DataPaths) -> std::io::Result<()> {
        let theme = if self.theme.is_empty() { "auto" } else { &self.theme };
        let v = json!({ "X": self.x, "Y": self.y, "W": self.w, "H": self.h, "Max": self.max, "Scale": self.scale, "Page": self.page, "Theme": theme });
        write(&paths.window(), &v)
    }

    /// Size recalculated when the display scale differs from the one it was saved at.
    pub fn scaled_size(&self, scale_now: f64) -> (i32, i32) {
        let k = if self.scale > 0.0 { scale_now / self.scale } else { 1.0 };
        ((f64::from(self.w) * k) as i32, (f64::from(self.h) * k) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_window_file() {
        let m: Map<String, Value> = serde_json::from_str(r#"{"X":10,"Y":20,"W":1240,"H":800,"Skala":1.25,"Strona":"analiza"}"#).unwrap();
        let p = WindowPrefs::from_map(&renamed(m, OLD_KEYS));
        assert_eq!(p.page, "analysis");
        assert_eq!(p.scaled_size(1.5), (1488, 960));
    }
}
