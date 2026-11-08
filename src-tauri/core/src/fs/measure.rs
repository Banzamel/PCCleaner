use std::path::{Path, PathBuf};

use super::exclude::Excluded;
use super::{age_cutoff, is_newer, link, lock, walk};

/// What a cleanup of some paths would delete.
#[derive(Debug, Clone, Default)]
pub struct Measured {
    /// Bytes [`super::delete::remove_folder_content`] will actually delete.
    pub bytes: f64,
    pub files: u64,
    /// Files that cannot be deleted now (in use) and their total size.
    pub in_use: f64,
    pub in_use_files: Vec<(String, u64)>,
}

/// Measures folders (their contents) and single files. Files newer than `min_age_days` are not part
/// of the category and are not counted at all.
pub fn measure_paths(paths: &[PathBuf], min_age_days: u32, excluded: &Excluded) -> Measured {
    let cutoff = age_cutoff(min_age_days);
    let mut m = Measured::default();
    for p in paths {
        if excluded.contains(p) {
            continue;
        }
        let Ok(md) = std::fs::symlink_metadata(p) else { continue };
        if link::is_link(&md) {
            continue;
        }
        if md.is_file() {
            add(&mut m, p, md.len(), md.modified().ok(), cutoff);
            continue;
        }
        for f in walk::files(p, excluded) {
            add(&mut m, &f.path, f.len, f.modified, cutoff);
        }
    }
    m
}

fn add(m: &mut Measured, path: &Path, len: u64, modified: Option<std::time::SystemTime>, cutoff: Option<std::time::SystemTime>) {
    if is_newer(modified, cutoff) {
        return;
    }
    if lock::is_locked(path) {
        m.in_use += len as f64;
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        m.in_use_files.push((name, len));
        return;
    }
    m.bytes += len as f64;
    m.files += 1;
}

/// Plain total of a folder: bytes, newest change and whether it holds `.exe` files.
#[derive(Debug, Clone, Default)]
pub struct FolderSummary {
    pub bytes: f64,
    pub last_write: Option<std::time::SystemTime>,
    pub has_exe: bool,
}

pub fn summarize_folder(path: &Path, excluded: &Excluded) -> FolderSummary {
    let mut s = FolderSummary::default();
    for f in walk::files(path, excluded) {
        s.bytes += f.len as f64;
        if f.modified > s.last_write {
            s.last_write = f.modified;
        }
        if !s.has_exe && f.path.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")) {
            s.has_exe = true;
        }
    }
    if s.last_write.is_none() {
        // Empty or unreadable folder: its own date, or "now" when even that is unknown, so an
        // unknown folder never enters the plan as old
        s.last_write = std::fs::metadata(path).and_then(|m| m.modified()).ok().or_else(|| Some(std::time::SystemTime::now()));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_files_and_single_paths() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join("a")).unwrap();
        std::fs::write(d.path().join("a/x"), vec![0u8; 100]).unwrap();
        std::fs::write(d.path().join("y"), vec![0u8; 50]).unwrap();
        let m = measure_paths(&[d.path().join("a"), d.path().join("y"), d.path().join("missing")], 0, &Excluded::default());
        assert_eq!(m.bytes, 150.0);
        assert_eq!(m.files, 2);
    }

    #[test]
    fn new_files_do_not_count_with_an_age_rule() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("fresh.log"), vec![0u8; 10]).unwrap();
        assert_eq!(measure_paths(&[d.path().to_path_buf()], 7, &Excluded::default()).files, 0);
    }
}
