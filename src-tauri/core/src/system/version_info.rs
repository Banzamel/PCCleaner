use std::path::Path;

/// "CompanyName" from an executable's version resource (the publisher shown for startup apps).
#[cfg(windows)]
pub fn company_name(path: &Path) -> Option<String> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
    let file = super::wide(&path.to_string_lossy());
    // SAFETY: the buffer is sized by GetFileVersionInfoSizeW; pointers returned by VerQueryValueW
    // point into that buffer, which lives until the end of the function.
    unsafe {
        let size = GetFileVersionInfoSizeW(PCWSTR(file.as_ptr()), None);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        GetFileVersionInfoW(PCWSTR(file.as_ptr()), None, size, data.as_mut_ptr().cast()).ok()?;
        let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut len = 0u32;
        let tr = super::wide("\\VarFileInfo\\Translation");
        let mut code = "040904B0".to_string();
        if VerQueryValueW(data.as_ptr().cast(), PCWSTR(tr.as_ptr()), &mut ptr, &mut len).as_bool() && len >= 4 {
            let w = std::slice::from_raw_parts(ptr as *const u16, 2);
            code = format!("{:04X}{:04X}", w[0], w[1]);
        }
        let q = super::wide(&format!("\\StringFileInfo\\{code}\\CompanyName"));
        if !VerQueryValueW(data.as_ptr().cast(), PCWSTR(q.as_ptr()), &mut ptr, &mut len).as_bool() || len == 0 {
            return None;
        }
        let s = std::slice::from_raw_parts(ptr as *const u16, len as usize);
        let end = s.iter().position(|c| *c == 0).unwrap_or(s.len());
        let name = String::from_utf16_lossy(&s[..end]).trim().to_string();
        (!name.is_empty()).then_some(name)
    }
}

#[cfg(not(windows))]
pub fn company_name(_path: &Path) -> Option<String> {
    None
}
