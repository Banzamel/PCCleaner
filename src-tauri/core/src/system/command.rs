use std::process::{Command, Stdio};

/// Output of a system tool (docker, reg.exe, pnputil, DISM...).
#[derive(Debug, Clone, Default)]
pub struct ToolOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ToolOutput {
    pub fn ok(&self) -> bool {
        self.code == 0
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.stdout.lines().map(str::trim).filter(|l| !l.is_empty())
    }
}

/// A command without a console window (the program has none, and a flashing console would be odd).
pub fn hidden(exe: &str) -> Command {
    let mut c = Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c.stdin(Stdio::null());
    c
}

/// Runs a tool and waits for it. A tool that cannot start (not installed) gives code -1.
pub fn run(exe: &str, args: &[&str]) -> ToolOutput {
    match hidden(exe).args(args).output() {
        Ok(o) => ToolOutput {
            code: o.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&o.stdout).to_string(),
            stderr: String::from_utf8_lossy(&o.stderr).to_string(),
        },
        Err(e) => ToolOutput { code: -1, stdout: String::new(), stderr: e.to_string() },
    }
}

/// Whether a program can be found on PATH (e.g. docker).
pub fn on_path(exe: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else { return false };
    std::env::split_paths(&path).any(|d| d.join(exe).is_file() || d.join(format!("{exe}.exe")).is_file())
}

/// Full path of a Windows tool in System32 (independent of PATH).
pub fn system32(exe: &str) -> String {
    let windir = std::env::var("windir").unwrap_or_else(|_| "C:\\Windows".into());
    format!("{windir}\\System32\\{exe}")
}
