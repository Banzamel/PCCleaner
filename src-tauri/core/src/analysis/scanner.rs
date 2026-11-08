//! Fast whole-drive scan. It does not follow junctions or symbolic links and skips cloud-only files
//! (OneDrive). Hard links (WinSxS) are counted more than once, like in Explorer.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use crate::fs::link::{self, CLOUD_ONLY};

#[derive(Debug, Clone)]
pub struct FolderStat {
    pub path: String,
    pub bytes: u64,
    pub files: u64,
    pub last_write: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct FileStat {
    pub path: String,
    pub bytes: u64,
    pub last_write: Option<SystemTime>,
}

#[derive(Debug, Default)]
pub struct RawScan {
    pub folders: Vec<FolderStat>,
    pub files: Vec<FileStat>,
    pub bytes: u64,
    pub file_count: u64,
    pub denied: u64,
}

pub struct Limits {
    pub max_depth: usize,
    pub min_folder: u64,
    pub min_file: u64,
}

/// Scans `root`; `counter` counts files as they are seen, for the progress line.
pub fn scan(root: &Path, limits: &Limits, counter: &AtomicU64) -> RawScan {
    let mut r = RawScan::default();
    let (bytes, files, _) = walk(root, 0, limits, &mut r, counter);
    r.bytes = bytes;
    r.file_count = files;
    r
}

fn walk(dir: &Path, depth: usize, l: &Limits, r: &mut RawScan, counter: &AtomicU64) -> (u64, u64, Option<SystemTime>) {
    let (mut total, mut files, mut last) = (0u64, 0u64, None);
    let Ok(rd) = std::fs::read_dir(dir) else {
        r.denied += 1;
        return (0, 0, None);
    };
    for e in rd.flatten() {
        let Ok(md) = e.metadata() else { continue };
        if link::is_link(&md) {
            continue;
        }
        if md.is_dir() {
            let (b, f, t) = walk(&e.path(), depth + 1, l, r, counter);
            total += b;
            files += f;
            last = last.max(t);
            continue;
        }
        if link::attributes(&md) & CLOUD_ONLY != 0 {
            continue;
        }
        let (b, t) = (md.len(), md.modified().ok());
        total += b;
        files += 1;
        counter.fetch_add(1, Ordering::Relaxed);
        last = last.max(t);
        if b >= l.min_file {
            r.files.push(FileStat { path: e.path().to_string_lossy().to_string(), bytes: b, last_write: t });
        }
    }
    if depth >= 1 && depth <= l.max_depth && total >= l.min_folder {
        r.folders.push(FolderStat { path: dir.to_string_lossy().to_string(), bytes: total, files, last_write: last });
    }
    (total, files, last)
}

/// The most specific large folders: those without a large sub-folder, or with a large "rest" of
/// files outside their large sub-folders (then only the rest is counted).
pub fn most_specific(folders: &[FolderStat], threshold: u64) -> Vec<(FolderStat, bool)> {
    let parent = |p: &str| p.rfind(['\\', '/']).map(|i| p[..i].to_string());
    let mut children: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let known: std::collections::HashSet<&str> = folders.iter().map(|f| f.path.as_str()).collect();
    for f in folders {
        if let Some(p) = parent(&f.path).filter(|p| known.contains(p.as_str())) {
            *children.entry(p).or_default() += f.bytes;
        }
    }
    folders
        .iter()
        .filter_map(|f| {
            let c = children.get(&f.path).copied().unwrap_or(0);
            if c == 0 {
                Some((f.clone(), false))
            } else if f.bytes - c >= threshold {
                Some((FolderStat { bytes: f.bytes - c, ..f.clone() }, true))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(p: &str, b: u64) -> FolderStat {
        FolderStat { path: p.into(), bytes: b, files: 1, last_write: None }
    }

    #[test]
    fn keeps_leaves_and_large_remainders() {
        let all = vec![f("C:\\Games", 10), f("C:\\Games\\A", 6), f("C:\\Games\\B", 3), f("C:\\Data", 5), f("C:\\Data\\X", 4)];
        let r = most_specific(&all, 1);
        let names: Vec<_> = r.iter().map(|(x, rest)| (x.path.as_str(), x.bytes, *rest)).collect();
        assert!(names.contains(&("C:\\Games", 1, true)));
        assert!(names.contains(&("C:\\Games\\A", 6, false)));
        assert!(names.contains(&("C:\\Data", 1, true)));
        assert_eq!(most_specific(&all, 2).len(), 3);
    }

    #[test]
    fn scans_a_tree() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("a/b")).unwrap();
        std::fs::write(d.path().join("a/b/big"), vec![0u8; 2000]).unwrap();
        std::fs::write(d.path().join("small"), vec![0u8; 10]).unwrap();
        let c = AtomicU64::new(0);
        let r = scan(d.path(), &Limits { max_depth: 6, min_folder: 1000, min_file: 1000 }, &c);
        assert_eq!((r.bytes, r.file_count, r.files.len(), r.folders.len()), (2010, 2, 1, 2));
    }
}
