use std::path::{Path, PathBuf};

use super::exclude::Excluded;
use super::{age_cutoff, is_newer, link};

/// Result of deleting a folder's contents. Every file that could not be deleted is listed
/// (capped, the count stays exact), so the log tells the user what was skipped and why.
#[derive(Debug, Clone, Default)]
pub struct DeleteOutcome {
    pub bytes: f64,
    pub errors: u64,
    pub failures: Vec<String>,
}

const MAX_FAILURES: usize = 200;

impl DeleteOutcome {
    fn fail(&mut self, path: &Path, err: impl std::fmt::Display) {
        self.errors += 1;
        if self.failures.len() < MAX_FAILURES {
            self.failures.push(format!("{}: {err}", path.display()));
        }
    }

    pub fn merge(&mut self, other: DeleteOutcome) {
        self.bytes += other.bytes;
        self.errors += other.errors;
        let room = MAX_FAILURES.saturating_sub(self.failures.len());
        self.failures.extend(other.failures.into_iter().take(room));
    }
}

/// Deletes the CONTENTS of a folder (not the folder itself) or a single file.
/// - links (junctions, symbolic links) inside are deleted as links; their targets are never entered,
/// - a root that is itself a link is left alone,
/// - files in use fail and are counted, the rest continues,
/// - with `min_age_days > 0` only files older than that are deleted - checked for EVERY file,
/// - excluded folders are skipped, also deep inside.
pub fn remove_folder_content(path: &Path, min_age_days: u32, excluded: &Excluded) -> DeleteOutcome {
    let mut out = DeleteOutcome::default();
    if excluded.contains(path) {
        return out;
    }
    let Ok(md) = std::fs::symlink_metadata(path) else { return out };
    if link::is_link(&md) {
        return out;
    }
    if md.is_file() {
        delete_file(path, md.len(), link::attributes(&md), &mut out);
        return out;
    }
    let cutoff = age_cutoff(min_age_days);
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) => {
                out.fail(&dir, e);
                continue;
            }
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let Ok(md) = entry.metadata() else { continue };
            if link::is_link(&md) {
                // A link is removed as a link only when everything goes; its target is never touched
                if cutoff.is_none() {
                    if let Err(e) = remove_link(&p) {
                        out.fail(&p, e);
                    }
                }
                continue;
            }
            if md.is_dir() {
                if excluded.contains(&p) {
                    continue;
                }
                if !is_newer(md.modified().ok(), cutoff) {
                    dirs.push(p.clone());
                }
                stack.push(p);
                continue;
            }
            if is_newer(md.modified().ok(), cutoff) {
                continue;
            }
            delete_file(&p, md.len(), link::attributes(&md), &mut out);
        }
    }
    // Empty folders, deepest first; non-empty ones (files in use, recent files) stay
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for d in dirs {
        let _ = std::fs::remove_dir(&d);
    }
    out
}

fn delete_file(path: &Path, len: u64, attrs: u32, out: &mut DeleteOutcome) {
    if attrs & link::READONLY != 0 {
        if let Ok(md) = std::fs::metadata(path) {
            let mut perm = md.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perm.set_readonly(false);
            let _ = std::fs::set_permissions(path, perm);
        }
    }
    match std::fs::remove_file(path) {
        Ok(()) => out.bytes += len as f64,
        Err(e) => out.fail(path, e),
    }
}

/// Removes a junction or symbolic link itself. A directory link needs remove_dir on Windows,
/// a file link remove_file; neither follows the link.
fn remove_link(path: &Path) -> std::io::Result<()> {
    std::fs::remove_dir(path).or_else(|_| std::fs::remove_file(path))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn removes_contents_but_not_link_targets() {
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("keep.txt"), b"precious").unwrap();
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("a/b")).unwrap();
        std::fs::write(root.path().join("a/b/x.tmp"), vec![0u8; 10]).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("a/link")).unwrap();
        let out = remove_folder_content(root.path(), 0, &Excluded::default());
        assert_eq!(out.bytes, 10.0);
        assert!(outside.path().join("keep.txt").exists(), "link target must survive");
        assert!(!root.path().join("a").exists());
        assert!(root.path().exists(), "the folder itself stays");
    }

    #[test]
    fn root_link_is_left_alone() {
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("keep.txt"), b"x").unwrap();
        let root = tempfile::tempdir().unwrap();
        let link = root.path().join("cache");
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();
        remove_folder_content(&link, 0, &Excluded::default());
        assert!(outside.path().join("keep.txt").exists());
    }

    #[test]
    fn age_rule_keeps_new_files_and_excluded_folders_stay() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("new.log"), b"x").unwrap();
        std::fs::create_dir(root.path().join("keep")).unwrap();
        std::fs::write(root.path().join("keep/a"), b"x").unwrap();
        let ex = Excluded::new(&[root.path().join("keep").to_string_lossy().to_string()]);
        remove_folder_content(root.path(), 7, &ex);
        assert!(root.path().join("new.log").exists());
        remove_folder_content(root.path(), 0, &ex);
        assert!(!root.path().join("new.log").exists());
        assert!(root.path().join("keep/a").exists());
    }

    #[test]
    fn reports_failures_per_item() {
        let mut out = DeleteOutcome::default();
        out.fail(Path::new("/x/y"), "denied");
        assert_eq!(out.errors, 1);
        assert_eq!(out.failures, vec!["/x/y: denied".to_string()]);
    }
}
