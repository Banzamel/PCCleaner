//! Administrator rights. The program needs them for every mode (system caches, other drives,
//! registry), so a normal start relaunches itself through UAC, exactly like the old launcher did.

#[cfg(windows)]
pub fn is_admin() -> bool {
    // SAFETY: no arguments; returns whether the token is in the Administrators group and elevated.
    unsafe { windows::Win32::UI::Shell::IsUserAnAdmin().as_bool() }
}

/// SID of the account this process runs as (S-1-5-21-...).
#[cfg(windows)]
pub fn current_sid() -> Option<String> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree};
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    // SAFETY: token handle and the LocalAlloc'd string are released; the buffer is sized by the first call.
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).ok()?;
        let mut len = 0u32;
        let _ = GetTokenInformation(token, TokenUser, None, 0, &mut len);
        let mut buf = vec![0u8; len as usize];
        let r = GetTokenInformation(token, TokenUser, Some(buf.as_mut_ptr().cast()), len, &mut len);
        let _ = CloseHandle(token);
        r.ok()?;
        let user = &*(buf.as_ptr() as *const TOKEN_USER);
        let mut s = PWSTR::null();
        ConvertSidToStringSidW(user.User.Sid, &mut s).ok()?;
        let out = s.to_string().ok();
        let _ = LocalFree(Some(HLOCAL(s.0.cast())));
        out
    }
}

/// Starts this program again through UAC ("runas"). Returns false when the user declined.
#[cfg(windows)]
pub fn relaunch_elevated(args: &[String]) -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW};
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let Ok(exe) = std::env::current_exe() else { return false };
    let params = args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" ");
    let (verb, file, par) = (super::wide("runas"), super::wide(&exe.to_string_lossy()), super::wide(&params));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(par.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    // SAFETY: all strings outlive the call; the struct size is set.
    unsafe { ShellExecuteExW(&mut info).is_ok() }
}

#[cfg(not(windows))]
pub fn is_admin() -> bool {
    true
}
#[cfg(not(windows))]
pub fn current_sid() -> Option<String> {
    None
}
#[cfg(not(windows))]
pub fn relaunch_elevated(_args: &[String]) -> bool {
    false
}

/// Quotes one command-line argument the way CommandLineToArgvW reads it back.
pub fn quote(a: &str) -> String {
    if !a.is_empty() && !a.contains([' ', '\t', '"']) {
        return a.to_string();
    }
    let mut out = String::from("\"");
    let mut slashes = 0;
    for c in a.chars() {
        match c {
            '\\' => slashes += 1,
            '"' => {
                out.push_str(&"\\".repeat(slashes * 2 + 1));
                out.push('"');
                slashes = 0;
            }
            _ => {
                out.push_str(&"\\".repeat(slashes));
                out.push(c);
                slashes = 0;
            }
        }
    }
    out.push_str(&"\\".repeat(slashes * 2));
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn quoting() {
        assert_eq!(super::quote("C,P"), "C,P");
        assert_eq!(super::quote("a b"), "\"a b\"");
        assert_eq!(super::quote("C:\\x y\\"), "\"C:\\x y\\\\\"");
    }
}
