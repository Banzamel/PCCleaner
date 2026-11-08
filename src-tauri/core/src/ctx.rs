use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::fs::exclude::Excluded;
use crate::report::Reporter;
use crate::user::UserContext;

/// Everything a scan or cleanup step needs: whose profile, what to skip, where to report.
pub struct Ctx<'a> {
    pub user: &'a UserContext,
    pub excluded: &'a Excluded,
    pub out: &'a dyn Reporter,
    /// Folder of PCCleaner.exe - never offered as a leftover.
    pub app_dir: PathBuf,
    /// Drives ticked on the Overview, as `C:`; the system drive is always among them.
    pub drives: Vec<String>,
    errors: AtomicUsize,
}

impl<'a> Ctx<'a> {
    pub fn new(user: &'a UserContext, excluded: &'a Excluded, out: &'a dyn Reporter, app_dir: PathBuf, drives: Vec<String>) -> Self {
        Self { user, excluded, out, app_dir, drives, errors: AtomicUsize::new(0) }
    }

    /// An item that cannot be read (odd folder, registry entry, broken shortcut) is skipped instead
    /// of aborting the whole scan; the details go to the log.
    pub fn skip(&self, what: &str, err: impl std::fmt::Display) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        self.out.log(&format!("Skipped ({what}): {err}"));
    }

    pub fn errors(&self) -> usize {
        self.errors.load(Ordering::Relaxed)
    }

    /// Whether a path lies on one of the ticked drives.
    pub fn on_selected_drive(&self, path: &str) -> bool {
        path.len() >= 2 && path.is_char_boundary(2) && self.drives.iter().any(|d| d.eq_ignore_ascii_case(&path[..2]))
    }
}
