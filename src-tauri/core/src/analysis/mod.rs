//! "What takes up space": a read-only view of the largest folders and files on a drive. Nothing from
//! it ever enters the plan or gets deleted.

pub mod hints;
pub mod scanner;
pub mod shadow;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::report::Reporter;
use crate::scan::leftover::date;
use crate::system::drives;
use crate::text::Text;

const GB: u64 = 1 << 30;
const TB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisItem {
    /// A1, A2...
    pub id: String,
    pub kind: &'static str,
    pub path: String,
    pub bytes: f64,
    /// Only the files outside the listed sub-folders are counted.
    pub remainder: bool,
    pub files: u64,
    pub last_write: Option<String>,
    pub hint: Option<Text>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub letter: String,
    pub size: f64,
    pub free: f64,
    pub scanned_bytes: f64,
    pub folders: Vec<AnalysisItem>,
    pub files: Vec<AnalysisItem>,
    /// Restore points and shadow copies - invisible to the scanner (System Volume Information).
    pub shadow_copies: f64,
    pub threshold: f64,
}

fn day(t: Option<std::time::SystemTime>) -> Option<String> {
    t.map(date).filter(|d| d.as_str() > "1980-01-01")
}

/// Scans a drive (`C`), usually 30 s - 3 min, with a progress line every 200 ms.
pub fn analyse(letter: &str, system_drive: &str, out: &dyn Reporter) -> Analysis {
    let drive = format!("{letter}:");
    let info = drives::fixed_drives(system_drive).into_iter().find(|d| d.letter.eq_ignore_ascii_case(&drive));
    let (size, free) = info.map(|d| (d.size, d.free)).unwrap_or((0.0, 0.0));
    let threshold = if size - free > TB { 2 * GB } else { GB };
    let counter = AtomicU64::new(0);
    let done = AtomicBool::new(false);
    let started = Instant::now();
    let limits = scanner::Limits { max_depth: 6, min_folder: threshold, min_file: GB };
    let raw = std::thread::scope(|s| {
        let job = std::thread::Builder::new().stack_size(64 << 20).spawn_scoped(s, || {
            let r = scanner::scan(std::path::Path::new(&format!("{drive}\\")), &limits, &counter);
            done.store(true, Ordering::Release);
            r
        });
        let Ok(job) = job else { return scanner::RawScan::default() };
        while !done.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_millis(200));
            out.status(Text::new("scanned {0:N0} files ({1:N0} s)...").arg(counter.load(Ordering::Relaxed)).arg(started.elapsed().as_secs()));
        }
        job.join().unwrap_or_default()
    });
    out.status(Text::new("scanned {0:N0} files in {1:N0} s.").arg(raw.file_count).arg(started.elapsed().as_secs()));

    let mut folders: Vec<_> = scanner::most_specific(&raw.folders, threshold).into_iter().collect();
    folders.sort_by(|a, b| b.0.bytes.cmp(&a.0.bytes));
    let mut files = raw.files.clone();
    files.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let mut n = 0;
    let mut next = || {
        n += 1;
        format!("A{n}")
    };
    let folders: Vec<AnalysisItem> = folders
        .into_iter()
        .take(20)
        .map(|(f, rest)| AnalysisItem { id: next(), kind: "folder", hint: hints::hint(&f.path), path: f.path, bytes: f.bytes as f64, remainder: rest, files: f.files, last_write: day(f.last_write) })
        .collect();
    let files: Vec<AnalysisItem> = files
        .into_iter()
        .take(10)
        .map(|f| AnalysisItem { id: next(), kind: "file", hint: hints::hint(&f.path), path: f.path, bytes: f.bytes as f64, remainder: false, files: 1, last_write: day(f.last_write) })
        .collect();
    out.log(&format!("Analysis {letter}: {} files, {} folders, {} files >1GB, access denied: {}", raw.file_count, folders.len(), files.len(), raw.denied));
    Analysis { letter: letter.to_string(), size, free, scanned_bytes: raw.bytes as f64, folders, files, shadow_copies: shadow::used_on(&drive), threshold: threshold as f64 }
}
