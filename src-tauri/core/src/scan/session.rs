use serde::Serialize;

use super::category::{measurable, Category, Kind, Measurement};
use super::leftover::Leftover;
use super::profile::ProfileScan;
use crate::text::Text;

/// A measured category, kept until the cleanup so it cleans exactly what was measured.
pub struct ScannedCategory {
    pub cat: Box<dyn Category>,
    pub m: Measurement,
    pub blocker: Option<Text>,
}

/// The result of a full scan. The UI only gets [`ScanReport`]; cleaning refers to items by id, so
/// nothing the window sends can add a path to delete.
#[derive(Default)]
pub struct ScanSession {
    pub categories: Vec<ScannedCategory>,
    pub leftovers: Vec<Leftover>,
    pub profile: ProfileScan,
    pub free_before: f64,
    pub errors: usize,
    pub drives: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryView {
    pub id: String,
    pub name: Text,
    pub description: Text,
    pub kind: Kind,
    pub opt_in: bool,
    pub bytes: f64,
    pub files: u64,
    pub locked: f64,
    pub note: Option<Text>,
    pub blocker: Option<Text>,
    /// The category is skipped while some program runs (browsers).
    pub has_processes: bool,
    pub measurable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub categories: Vec<CategoryView>,
    pub leftovers: Vec<Leftover>,
    pub profile: ProfileScan,
    pub free_before: f64,
    pub scan_errors: usize,
    pub drives: Vec<String>,
}

impl ScanSession {
    pub fn report(&self) -> ScanReport {
        let categories = self
            .categories
            .iter()
            .map(|c| CategoryView {
                id: c.cat.id().to_string(),
                name: c.cat.name(),
                description: c.cat.description(),
                kind: c.cat.kind(),
                opt_in: c.cat.opt_in(),
                bytes: c.m.bytes,
                files: c.m.files,
                locked: c.m.locked,
                note: c.m.note.clone(),
                blocker: c.blocker.clone(),
                has_processes: !c.cat.blocking_processes().is_empty(),
                measurable: measurable(c.cat.kind(), c.m.bytes),
            })
            .collect();
        ScanReport {
            categories,
            leftovers: self.leftovers.clone(),
            profile: self.profile.clone(),
            free_before: self.free_before,
            scan_errors: self.errors,
            drives: self.drives.clone(),
        }
    }
}
