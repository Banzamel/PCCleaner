use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

/// One log file per run: `logs\PCCleaner_yyyy-MM-dd_HH-mm.log`, lines `[HH:mm:ss] text`.
/// Logs live outside the program folder, so they survive updates.
pub struct Logger {
    pub path: PathBuf,
    file: Mutex<Option<File>>,
}

impl Logger {
    pub fn new(dir: &Path) -> Self {
        let _ = std::fs::create_dir_all(dir);
        let path = dir.join(format!("PCCleaner_{}.log", super::paths::stamp()));
        let file = OpenOptions::new().create(true).append(true).open(&path).ok();
        Self { path, file: Mutex::new(file) }
    }

    pub fn write(&self, text: &str) {
        let line = format!("[{}] {}\n", chrono::Local::now().format("%H:%M:%S"), text);
        if let Ok(mut f) = self.file.lock() {
            if let Some(file) = f.as_mut() {
                let _ = file.write_all(line.as_bytes());
            }
        }
    }
}

/// Deletes run logs older than `days` (all of them with 0), never the current one.
/// Registry backups (a subfolder) are kept. Returns how many were deleted.
pub fn remove_old(dir: &Path, current: &Path, days: u32) -> usize {
    let cutoff = SystemTime::now().checked_sub(Duration::from_secs(u64::from(days) * 86_400));
    let Ok(rd) = std::fs::read_dir(dir) else { return 0 };
    let mut n = 0;
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        let is_log = name.starts_with("PCCleaner_") && name.ends_with(".log");
        if !is_log || p == current || !p.is_file() {
            continue;
        }
        let old = days == 0 || e.metadata().and_then(|m| m.modified()).ok().zip(cutoff).is_some_and(|(m, c)| m < c);
        if old && std::fs::remove_file(&p).is_ok() {
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_other_logs_but_not_the_current_one() {
        let d = tempfile::tempdir().unwrap();
        let log = Logger::new(d.path());
        log.write("hello");
        std::fs::write(d.path().join("PCCleaner_2020-01-01_10-00.log"), "x").unwrap();
        std::fs::write(d.path().join("other.txt"), "x").unwrap();
        assert_eq!(remove_old(d.path(), &log.path, 0), 1);
        assert!(log.path.exists());
        assert!(std::fs::read_to_string(&log.path).unwrap().contains("] hello"));
    }
}
