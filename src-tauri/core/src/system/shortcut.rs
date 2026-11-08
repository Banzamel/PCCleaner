use std::path::Path;

/// Target of a `.lnk` shortcut. Empty for MSI-advertised shortcuts and Store apps, like WScript.Shell
/// returned in the old program. The caller must hold a [`super::ComGuard`].
#[cfg(windows)]
pub fn target(lnk: &Path) -> Option<String> {
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ};
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, SLGP_RAWPATH};
    let file = super::wide(&lnk.to_string_lossy());
    // SAFETY: COM objects are released by their wrappers; the buffer length is passed to GetPath.
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;
        persist.Load(PCWSTR(file.as_ptr()), STGM_READ).ok()?;
        let mut buf = [0u16; 1024];
        link.GetPath(&mut buf, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32).ok()?;
        let len = buf.iter().position(|c| *c == 0).unwrap_or(0);
        let raw = String::from_utf16_lossy(&buf[..len]);
        Some(super::registry::expand_env(&raw))
    }
}

#[cfg(not(windows))]
pub fn target(_lnk: &Path) -> Option<String> {
    None
}
