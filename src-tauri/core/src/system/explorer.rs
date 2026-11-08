use std::path::Path;

use super::command;

/// Shows a file or folder selected in File Explorer.
pub fn reveal(path: &Path) {
    let arg = format!("/select,\"{}\"", path.display());
    let _ = spawn_raw("explorer.exe", &arg);
}

/// Opens a folder, a document, an `.msc` console or an `ms-settings:` page with its default handler.
#[cfg(windows)]
pub fn shell_open(target: &str) -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let (verb, file) = (super::wide("open"), super::wide(target));
    // SAFETY: strings outlive the call; return values above 32 mean success.
    let h = unsafe { ShellExecuteW(None, PCWSTR(verb.as_ptr()), PCWSTR(file.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
    h.0 as isize > 32
}

#[cfg(not(windows))]
pub fn shell_open(_target: &str) -> bool {
    false
}

/// Opens a text file in Notepad (the log after cleaning).
pub fn notepad(path: &Path) {
    let _ = command::hidden("notepad.exe").arg(path).spawn();
}

/// explorer.exe parses its own command line, so the argument is passed verbatim.
fn spawn_raw(exe: &str, arg: &str) -> std::io::Result<()> {
    let mut c = std::process::Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.raw_arg(arg);
    }
    #[cfg(not(windows))]
    c.arg(arg);
    c.spawn().map(|_| ())
}
