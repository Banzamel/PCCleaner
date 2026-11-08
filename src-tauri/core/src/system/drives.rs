use serde::Serialize;

/// A local fixed drive with a letter.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    /// `C:`
    pub letter: String,
    pub label: String,
    pub size: f64,
    pub free: f64,
    pub system: bool,
}

/// Local fixed drives with a letter. Skipped: USB drives and cards, network and CD drives, partitions
/// without a letter (EFI, Recovery) and unavailable volumes, e.g. locked by BitLocker (no size).
#[cfg(windows)]
pub fn fixed_drives(system_drive: &str) -> Vec<DriveInfo> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW};
    const DRIVE_FIXED: u32 = 3;
    let mut out = Vec::new();
    // SAFETY: plain bitmask query.
    let mask = unsafe { GetLogicalDrives() };
    for i in 0..26u32 {
        if mask & (1 << i) == 0 {
            continue;
        }
        let letter = format!("{}:", (b'A' + i as u8) as char);
        let root = super::wide(&format!("{letter}\\"));
        // SAFETY: root is a valid NUL-terminated path; out parameters are local.
        unsafe {
            if GetDriveTypeW(PCWSTR(root.as_ptr())) != DRIVE_FIXED {
                continue;
            }
            let (mut free, mut total) = (0u64, 0u64);
            if GetDiskFreeSpaceExW(PCWSTR(root.as_ptr()), None, Some(&mut total), Some(&mut free)).is_err() || total == 0 {
                continue;
            }
            let mut name = [0u16; 261];
            let _ = GetVolumeInformationW(PCWSTR(root.as_ptr()), Some(&mut name), None, None, None, None);
            let len = name.iter().position(|c| *c == 0).unwrap_or(0);
            let system = letter.eq_ignore_ascii_case(system_drive);
            out.push(DriveInfo { label: String::from_utf16_lossy(&name[..len]), letter, size: total as f64, free: free as f64, system });
        }
    }
    out
}

#[cfg(not(windows))]
pub fn fixed_drives(_system_drive: &str) -> Vec<DriveInfo> {
    Vec::new()
}

/// Free bytes on a drive (`C:`), 0 when unknown.
pub fn free_space(letter: &str, system_drive: &str) -> f64 {
    fixed_drives(system_drive).into_iter().find(|d| d.letter.eq_ignore_ascii_case(letter)).map(|d| d.free).unwrap_or(0.0)
}
