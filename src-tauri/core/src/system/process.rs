use std::path::PathBuf;

/// A running process: lower-case name without `.exe` and, when readable, the full path.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub path: Option<PathBuf>,
}

#[cfg(windows)]
pub fn list() -> Vec<ProcessInfo> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
    let mut out = Vec::new();
    // SAFETY: snapshot handle is closed below; the entry struct has its size field set as required.
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return out };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        if Process32FirstW(snap, &mut e).is_ok() {
            loop {
                let len = e.szExeFile.iter().position(|c| *c == 0).unwrap_or(e.szExeFile.len());
                let exe = String::from_utf16_lossy(&e.szExeFile[..len]).to_lowercase();
                let name = exe.strip_suffix(".exe").unwrap_or(&exe).to_string();
                out.push(ProcessInfo { name, path: image_path(e.th32ProcessID) });
                if Process32NextW(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    out
}

#[cfg(windows)]
fn image_path(pid: u32) -> Option<PathBuf> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
    if pid == 0 {
        return None;
    }
    // SAFETY: the process handle is closed before returning; the buffer length is passed in.
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let r = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = CloseHandle(h);
        r.ok()?;
        Some(PathBuf::from(String::from_utf16_lossy(&buf[..len as usize])))
    }
}

#[cfg(not(windows))]
pub fn list() -> Vec<ProcessInfo> {
    Vec::new()
}

/// Names from `names` (lower case, without .exe) that are running now.
pub fn running(names: &[&str]) -> Vec<String> {
    if names.is_empty() {
        return Vec::new();
    }
    let all = list();
    let mut found: Vec<String> = names.iter().filter(|n| all.iter().any(|p| p.name == **n)).map(|n| n.to_string()).collect();
    found.dedup();
    found
}

pub fn is_running(names: &[&str]) -> bool {
    !running(names).is_empty()
}

/// Ends every process with this name (File Explorer before the thumbnail cache is deleted).
pub fn kill(name: &str) {
    let image = format!("{name}.exe");
    super::command::run(&super::command::system32("taskkill.exe"), &["/F", "/IM", &image]);
}
