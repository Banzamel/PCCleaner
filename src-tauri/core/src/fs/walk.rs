use std::fs::ReadDir;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::exclude::Excluded;
use super::link;

/// A file found by [`files`].
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub len: u64,
    pub modified: Option<SystemTime>,
    pub attrs: u32,
}

/// All files under a folder, depth first. Junctions and symbolic links are skipped (never entered),
/// excluded folders are skipped, unreadable folders are passed over. A root that is itself a link
/// yields nothing.
pub fn files<'a>(root: &Path, excluded: &'a Excluded) -> Files<'a> {
    let start = match std::fs::symlink_metadata(root) {
        Ok(md) if md.is_dir() && !link::is_link(&md) && !excluded.contains(root) => vec![root.to_path_buf()],
        _ => Vec::new(),
    };
    Files { stack: start, current: None, excluded }
}

pub struct Files<'a> {
    stack: Vec<PathBuf>,
    current: Option<ReadDir>,
    excluded: &'a Excluded,
}

impl Iterator for Files<'_> {
    type Item = FileEntry;

    fn next(&mut self) -> Option<FileEntry> {
        loop {
            if let Some(rd) = self.current.as_mut() {
                match rd.next() {
                    Some(Ok(e)) => {
                        // DirEntry::metadata does not follow links on Windows or Unix
                        let Ok(md) = e.metadata() else { continue };
                        if link::is_link(&md) {
                            continue;
                        }
                        let path = e.path();
                        if md.is_dir() {
                            if !self.excluded.contains(&path) {
                                self.stack.push(path);
                            }
                            continue;
                        }
                        return Some(FileEntry { path, len: md.len(), modified: md.modified().ok(), attrs: link::attributes(&md) });
                    }
                    Some(Err(_)) => continue,
                    None => self.current = None,
                }
            }
            let dir = self.stack.pop()?;
            self.current = std::fs::read_dir(&dir).ok();
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn does_not_enter_symlinks() {
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("secret.txt"), b"x").unwrap();
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("sub")).unwrap();
        std::fs::write(root.path().join("sub/a.txt"), b"abc").unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("link")).unwrap();
        let found: Vec<_> = files(root.path(), &Excluded::default()).map(|f| f.path).collect();
        assert_eq!(found, vec![root.path().join("sub/a.txt")]);
    }

    #[test]
    fn skips_excluded_folders() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("keep")).unwrap();
        std::fs::write(root.path().join("keep/a"), b"1").unwrap();
        std::fs::write(root.path().join("b"), b"1").unwrap();
        let ex = Excluded::new(&[root.path().join("keep").to_string_lossy().to_string()]);
        assert_eq!(files(root.path(), &ex).count(), 1);
    }
}
