use std::path::Path;

/// Whether a file cannot be deleted right now: opened by another program without delete sharing
/// (e.g. swap.vhdx of a running WSL) or no permission. It opens the file with DELETE access only,
/// like a delete does, so no data is read and antivirus does not scan it (fast on many files).
#[cfg(windows)]
pub fn is_locked(path: &Path) -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    const DELETE: u32 = 0x0001_0000;
    let wide = crate::system::wide(&crate::system::long_path(path));
    // SAFETY: `wide` is a NUL-terminated UTF-16 buffer that outlives the call.
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            DELETE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_OPEN_REPARSE_POINT,
            None,
        )
    };
    match handle {
        Ok(h) => {
            // SAFETY: the handle was just opened and is closed exactly once.
            let _ = unsafe { CloseHandle(h) };
            false
        }
        Err(_) => true,
    }
}

#[cfg(not(windows))]
pub fn is_locked(_path: &Path) -> bool {
    false
}
