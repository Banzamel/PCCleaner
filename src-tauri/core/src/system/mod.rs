//! Windows APIs and system tools. Every function has a `cfg(not(windows))` fallback that returns
//! "nothing found", so the rest of the core compiles and its pure logic is tested on any OS.

pub mod command;
pub mod drives;
pub mod elevation;
pub mod explorer;
pub mod job;
pub mod locale;
pub mod process;
pub mod recycle;
pub mod registry;
pub mod restore_point;
pub mod services;
pub mod shortcut;
pub mod tasks;
pub mod version_info;
pub mod wmi;

use std::path::Path;

/// NUL-terminated UTF-16 for Windows APIs.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// `\\?\` form of an absolute path, so paths longer than 260 characters work.
pub fn long_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.starts_with("\\\\?\\") {
        s.to_string()
    } else if let Some(unc) = s.strip_prefix("\\\\") {
        format!("\\\\?\\UNC\\{unc}")
    } else {
        format!("\\\\?\\{s}")
    }
}

/// Keeps COM initialised (multithreaded) on the current thread while it lives. Every blocking task
/// that touches COM (shortcuts, Task Scheduler, WMI, Windows Update) creates one.
pub struct ComGuard {
    #[cfg(windows)]
    owned: bool,
}

impl ComGuard {
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
            // SAFETY: plain COM initialisation; balanced by CoUninitialize in Drop when it succeeded.
            let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
            Self { owned: hr.is_ok() }
        }
        #[cfg(not(windows))]
        Self {}
    }
}

impl Default for ComGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        if self.owned {
            // SAFETY: matches the successful CoInitializeEx above on the same thread.
            unsafe { windows::Win32::System::Com::CoUninitialize() };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_paths() {
        assert_eq!(long_path(Path::new("C:\\a")), "\\\\?\\C:\\a");
        assert_eq!(long_path(Path::new("\\\\srv\\share")), "\\\\?\\UNC\\srv\\share");
    }
}
