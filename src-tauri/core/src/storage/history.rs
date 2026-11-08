use serde::Serialize;
use serde_json::{json, Value};

use super::json::{as_f64, as_items, as_string, read_value, write};
use super::paths::DataPaths;

/// One cleanup in history.json (the last 20 runs, newest first).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    /// yyyy-MM-dd HH:mm
    pub date: String,
    pub drive: String,
    pub total: f64,
    pub free_before: f64,
    pub free_after: f64,
}

const KEEP: usize = 20;

pub fn load(paths: &DataPaths) -> Vec<HistoryEntry> {
    let Some(v) = read_value(&paths.history()) else { return Vec::new() };
    as_items(v)
        .into_iter()
        .filter_map(|item| {
            let m = item.as_object()?;
            let n = |k: &str| m.get(k).and_then(as_f64).unwrap_or(0.0);
            Some(HistoryEntry {
                date: m.get("Date").and_then(as_string)?,
                drive: m.get("Drive").and_then(as_string).unwrap_or_default(),
                total: n("Total"),
                free_before: n("FreeBefore"),
                free_after: n("FreeAfter"),
            })
        })
        .collect()
}

pub fn add(paths: &DataPaths, entry: HistoryEntry) -> std::io::Result<()> {
    let mut list = vec![entry];
    list.extend(load(paths).into_iter().take(KEEP - 1));
    let v: Vec<Value> = list
        .iter()
        .map(|e| json!({ "Date": e.date, "Drive": e.drive, "Total": e.total, "FreeBefore": e.free_before, "FreeAfter": e.free_after }))
        .collect();
    write(&paths.history(), &Value::Array(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_object_file_and_limit() {
        let d = tempfile::tempdir().unwrap();
        let p = DataPaths { root: d.path().to_path_buf() };
        std::fs::write(p.history(), "\u{feff}{\"Date\":\"2026-10-20 10:00\",\"Total\":5}").unwrap();
        assert_eq!(load(&p).len(), 1);
        for i in 0..25 {
            add(&p, HistoryEntry { date: format!("d{i}"), drive: "C:".into(), total: 1.0, free_before: 0.0, free_after: 1.0 }).unwrap();
        }
        let all = load(&p);
        assert_eq!(all.len(), 20);
        assert_eq!(all[0].date, "d24");
    }
}
