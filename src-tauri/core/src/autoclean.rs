//! Automatic cleaning at sign-in (Settings, off by default). A Task Scheduler task of the signed-in
//! account starts the program with `--auto-clean` and the highest rights, so UAC does not ask. How often
//! it really cleans (every sign-in, daily, weekly) is decided at start by [`is_due`].

use chrono::NaiveDate;
use std::path::Path;

use crate::system::tasks;

pub const TASK_NAME: &str = "PC Cleaner - automatic cleaning";

/// Whether the cleaning should run now. `last` is the yyyy-MM-dd of the previous one.
pub fn is_due(mode: &str, last: &str, today: NaiveDate) -> bool {
    let last = NaiveDate::parse_from_str(last, "%Y-%m-%d").ok();
    match mode {
        "logon" => true,
        "daily" => last.is_none_or(|d| d < today),
        "weekly" => last.is_none_or(|d| d <= today - chrono::Duration::days(7)),
        _ => false,
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Task definition: at sign-in of `user` after 2 minutes (let the sign-in settle), highest rights,
/// on battery too, no time limit (after the cleaning the user may keep the window open).
pub fn task_xml(exe: &Path, user: &str) -> String {
    let dir = exe.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or_default();
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><Description>PC Cleaner: automatic cleaning of the recommended categories at sign-in (Settings).</Description></RegistrationInfo>
  <Triggers><LogonTrigger><Enabled>true</Enabled><UserId>{user}</UserId><Delay>PT2M</Delay></LogonTrigger></Triggers>
  <Principals><Principal id="Author"><UserId>{user}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>HighestAvailable</RunLevel></Principal></Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author"><Exec><Command>"{exe}"</Command><Arguments>--auto-clean</Arguments><WorkingDirectory>{dir}</WorkingDirectory></Exec></Actions>
</Task>"#,
        user = escape(user),
        exe = escape(&exe.to_string_lossy()),
        dir = escape(&dir),
    )
}

/// Account name of this process as DOMAIN\user.
pub fn current_account() -> String {
    let user = std::env::var("USERNAME").unwrap_or_default();
    match std::env::var("USERDOMAIN") {
        Ok(d) if !d.is_empty() => format!("{d}\\{user}"),
        _ => user,
    }
}

/// Creates, updates or removes the task to match the setting. Returns the error text on failure.
pub fn apply(mode: &str, exe: &Path) -> Result<(), String> {
    let path = format!("\\{TASK_NAME}");
    if matches!(mode, "logon" | "daily" | "weekly") {
        tasks::register_xml(TASK_NAME, &task_xml(exe, &current_account()))
    } else if tasks::exists(&path) {
        tasks::delete(&path)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn due_rules() {
        let today = d("2026-11-08");
        assert!(is_due("logon", "2026-11-08", today));
        assert!(!is_due("daily", "2026-11-08", today));
        assert!(is_due("daily", "2026-11-07", today));
        assert!(is_due("daily", "", today));
        assert!(!is_due("weekly", "2026-11-02", today));
        assert!(is_due("weekly", "2026-11-01", today));
        assert!(!is_due("off", "", today));
    }

    #[test]
    fn xml_is_escaped() {
        let x = task_xml(Path::new("C:\\A&B\\PCCleaner.exe"), "PC\\Jan");
        assert!(x.contains("\"C:\\A&amp;B\\PCCleaner.exe\""));
        assert!(x.contains("<UserId>PC\\Jan</UserId>"));
    }
}
