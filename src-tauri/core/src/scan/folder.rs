use std::path::PathBuf;
use std::time::Duration;

use super::category::{Category, CleanOutcome, Kind, Measurement};
use crate::ctx::Ctx;
use crate::fs::{delete, measure};
use crate::system::{process, services};
use crate::text::Text;

/// Caches of one program; skipped on its own while that program runs.
#[derive(Debug, Clone)]
pub struct AppGroup {
    pub name: String,
    pub processes: Vec<&'static str>,
    pub paths: Vec<PathBuf>,
}

/// A category that empties folders (and single files): the common case.
pub struct FolderCategory {
    pub id: &'static str,
    pub name: Text,
    pub description: Text,
    pub opt_in: bool,
    pub paths: Vec<PathBuf>,
    /// Only files older than this many days (system logs).
    pub min_age: u32,
    /// Services stopped while the folder is emptied (Windows Update).
    pub services: Vec<&'static str>,
    pub processes: Vec<&'static str>,
    /// When set, each group is skipped separately while its program runs.
    pub groups: Vec<AppGroup>,
}

impl FolderCategory {
    pub fn new(id: &'static str, name: &str, description: Text, paths: Vec<PathBuf>) -> Self {
        Self { id, name: Text::new(name), description, opt_in: false, paths, min_age: 0, services: Vec::new(), processes: Vec::new(), groups: Vec::new() }
    }
}

/// "Total X: Y can be deleted, Z is locked - files used by running programs (a, b, c and N more)".
pub fn locked_note(m: &measure::Measured) -> Option<Text> {
    if m.in_use <= 0.0 {
        return None;
    }
    let mut files = m.in_use_files.clone();
    files.sort_by(|a, b| b.1.cmp(&a.1));
    let names = files.iter().take(3).map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", ");
    let names = if files.len() > 3 { Text::new("{0} and {1} more").arg(&names).arg(files.len() - 3) } else { Text::new("{0}").arg(&names) };
    Some(
        Text::new("Total {0}: {1} can be deleted, {2} is locked - files used by running programs ({3}). Close them to delete the rest.")
            .size(m.bytes + m.in_use)
            .size(m.bytes)
            .size(m.in_use)
            .text(names),
    )
}

impl Category for FolderCategory {
    fn id(&self) -> &'static str {
        self.id
    }
    fn name(&self) -> Text {
        self.name.clone()
    }
    fn description(&self) -> Text {
        self.description.clone()
    }
    fn kind(&self) -> Kind {
        Kind::Folder
    }
    fn opt_in(&self) -> bool {
        self.opt_in
    }
    fn blocking_processes(&self) -> &[&'static str] {
        &self.processes
    }

    fn measure(&self, ctx: &Ctx) -> Measurement {
        let m = measure::measure_paths(&self.paths, self.min_age, ctx.excluded);
        let note = locked_note(&m);
        if note.is_some() {
            ctx.out.log(&format!("{}: locked {} bytes in {} files", self.id, m.in_use, m.in_use_files.len()));
        }
        Measurement { bytes: m.bytes, files: m.files, locked: m.in_use, note }
    }

    fn clean(&self, ctx: &Ctx) -> CleanOutcome {
        let mut out = CleanOutcome::default();
        for s in &self.services {
            services::stop(s);
        }
        if !self.services.is_empty() {
            std::thread::sleep(Duration::from_secs(2));
        }
        let mut paths = self.paths.clone();
        if !self.groups.is_empty() {
            paths.clear();
            for g in &self.groups {
                if process::is_running(&g.processes) {
                    out.skipped_apps.push(g.name.clone());
                } else {
                    paths.extend(g.paths.iter().cloned());
                }
            }
        }
        let mut total = delete::DeleteOutcome::default();
        for p in &paths {
            total.merge(delete::remove_folder_content(p, self.min_age, ctx.excluded));
        }
        for s in &self.services {
            services::start(s);
        }
        out.bytes = total.bytes;
        out.errors = total.errors;
        out.failures = total.failures;
        out
    }
}
