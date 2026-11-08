//! Walking, measuring and deleting files. Nothing here ever enters a junction or a symbolic link:
//! links are skipped while measuring and removed as links (never followed) while deleting.

pub mod delete;
pub mod exclude;
pub mod link;
pub mod lock;
pub mod measure;
pub mod protect;
pub mod walk;

use std::time::{Duration, SystemTime};

/// Cut-off time for "older than N days" rules; `None` when every file counts.
pub fn age_cutoff(min_age_days: u32) -> Option<SystemTime> {
    if min_age_days == 0 {
        return None;
    }
    SystemTime::now().checked_sub(Duration::from_secs(u64::from(min_age_days) * 86_400))
}

/// Whether a file or folder changed after the cut-off (and so is too new to delete).
pub fn is_newer(modified: Option<SystemTime>, cutoff: Option<SystemTime>) -> bool {
    match (modified, cutoff) {
        (Some(m), Some(c)) => m > c,
        // Unknown date with an age rule: treat as new - an unknown item must not be deleted as old
        (None, Some(_)) => true,
        _ => false,
    }
}
