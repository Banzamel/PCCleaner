use serde::Deserialize;

use crate::text::Text;

/// Result of asking Windows for a System Restore point.
#[derive(Debug, Clone)]
pub struct RestorePointResult {
    pub ok: bool,
    pub description: Text,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SystemRestore {
    sequence_number: u32,
    creation_time: String,
}

fn last_point() -> Option<SystemRestore> {
    let mut all: Vec<SystemRestore> = super::wmi::query(super::wmi::DEFAULT, "SELECT SequenceNumber, CreationTime FROM SystemRestore");
    all.sort_by_key(|p| p.sequence_number);
    all.pop()
}

/// "20261020143000.000000-000" (WMI time) -> "20.10 14:30" as the old program showed it.
pub fn short_wmi_time(t: &str) -> String {
    if t.len() < 12 || !t.is_char_boundary(12) {
        return "?".into();
    }
    format!("{}.{} {}:{}", &t[6..8], &t[4..6], &t[8..10], &t[10..12])
}

/// Creates a restore point before changes the Recycle Bin cannot undo (registry, tasks, drivers).
/// Windows skips it when one was made in the last 24 hours - that is reported, not treated as an error.
pub fn create(description: &str) -> RestorePointResult {
    let before = last_point();
    if let Err(e) = set_point(description) {
        return RestorePointResult {
            ok: false,
            description: Text::new("not created ({0}) - System Protection may be turned off").arg(e),
        };
    }
    let after = last_point();
    match (&before, &after) {
        (b, Some(a)) if b.as_ref().map(|b| b.sequence_number) != Some(a.sequence_number) => {
            RestorePointResult { ok: true, description: Text::new("created (no. {0})").arg(a.sequence_number) }
        }
        _ => {
            let when = before.map(|b| short_wmi_time(&b.creation_time)).unwrap_or_else(|| "?".into());
            RestorePointResult { ok: false, description: Text::new("skipped by Windows - one restore point per day (last: {0})").arg(when) }
        }
    }
}

#[cfg(windows)]
fn set_point(description: &str) -> Result<(), String> {
    use windows::Win32::System::Restore::{SRSetRestorePointW, BEGIN_SYSTEM_CHANGE, END_SYSTEM_CHANGE, MODIFY_SETTINGS, RESTOREPOINTINFOW, STATEMGRSTATUS};
    let mut info = RESTOREPOINTINFOW { dwEventType: BEGIN_SYSTEM_CHANGE, dwRestorePtType: MODIFY_SETTINGS, ..Default::default() };
    for (i, c) in description.encode_utf16().take(255).enumerate() {
        info.szDescription[i] = c;
    }
    let mut status = STATEMGRSTATUS::default();
    // SAFETY: both structs are fully initialised locals.
    if !unsafe { SRSetRestorePointW(&info, &mut status) }.as_bool() {
        let code = status.nStatus;
        return Err(format!("error {}", code.0));
    }
    info.dwEventType = END_SYSTEM_CHANGE;
    info.llSequenceNumber = status.llSequenceNumber;
    // SAFETY: as above; closes the change opened by BEGIN_SYSTEM_CHANGE.
    let _ = unsafe { SRSetRestorePointW(&info, &mut status) };
    Ok(())
}

#[cfg(not(windows))]
fn set_point(_description: &str) -> Result<(), String> {
    Err("not available".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn wmi_time() {
        assert_eq!(super::short_wmi_time("20261020143000.000000-000"), "20.10 14:30");
        assert_eq!(super::short_wmi_time("bad"), "?");
    }
}
