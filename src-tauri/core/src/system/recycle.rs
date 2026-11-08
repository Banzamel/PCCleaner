use std::path::Path;

/// Moves a file or folder to the Recycle Bin, like Explorer's Delete. A folder is moved as a whole,
/// so links inside it are moved as links and never followed. When the item is too large for the
/// Recycle Bin Windows asks before deleting it permanently (FOF_WANTNUKEWARNING) instead of doing it
/// silently.
#[cfg(windows)]
pub fn send_to_recycle_bin(path: &Path) -> Result<(), String> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{
        SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FOF_WANTNUKEWARNING, FO_DELETE, SHFILEOPSTRUCTW,
    };
    // The source list is double-NUL terminated
    let mut from: Vec<u16> = path.to_string_lossy().encode_utf16().collect();
    from.extend([0, 0]);
    let mut op = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: PCWSTR(from.as_ptr()),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI | FOF_WANTNUKEWARNING).0 as u16,
        ..Default::default()
    };
    // SAFETY: `from` outlives the call and is double-NUL terminated as required.
    let r = unsafe { SHFileOperationW(&mut op) };
    if r != 0 {
        return Err(format!("SHFileOperation error 0x{r:X}"));
    }
    if op.fAnyOperationsAborted.as_bool() || path.exists() {
        return Err("not moved to the Recycle Bin".into());
    }
    Ok(())
}

/// Empties the Recycle Bin on all drives for the current account, without asking.
#[cfg(windows)]
pub fn empty_recycle_bin() -> Result<(), String> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND};
    // SAFETY: no window, all drives.
    unsafe { SHEmptyRecycleBinW(None, PCWSTR::null(), SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND) }.map_err(|e| e.to_string())
}

#[cfg(not(windows))]
pub fn send_to_recycle_bin(_path: &Path) -> Result<(), String> {
    Err("the Recycle Bin is available only on Windows".into())
}

#[cfg(not(windows))]
pub fn empty_recycle_bin() -> Result<(), String> {
    Err("the Recycle Bin is available only on Windows".into())
}
