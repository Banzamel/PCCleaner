use super::names::command_path;
use crate::scan::leftover::{Confidence, Leftover, LeftoverKind};
use crate::system::registry;
use crate::text::Text;

/// An entry in Apps & features (Uninstall registry keys).
#[derive(Debug, Clone, Default)]
pub struct UninstallEntry {
    pub reg_key: String,
    pub display_name: String,
    pub publisher: String,
    pub install_location: String,
    pub display_icon: String,
    pub uninstall_string: String,
    pub system_component: bool,
    pub windows_installer: bool,
    /// KB
    pub estimated_size: Option<f64>,
    pub install_date: String,
}

pub fn read_all(hkcu: &str) -> Vec<UninstallEntry> {
    let roots = [
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string(),
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string(),
        format!("{hkcu}\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];
    let mut out = Vec::new();
    for root in roots {
        for k in registry::subkeys(&root) {
            let key = format!("{root}\\{k}");
            let s = |n: &str| registry::read_string(&key, n).unwrap_or_default();
            let display_name = s("DisplayName");
            if display_name.is_empty() {
                continue;
            }
            out.push(UninstallEntry {
                display_name,
                publisher: s("Publisher"),
                install_location: s("InstallLocation"),
                display_icon: s("DisplayIcon"),
                uninstall_string: s("UninstallString"),
                system_component: registry::read_dword(&key, "SystemComponent") == Some(1),
                windows_installer: registry::read_dword(&key, "WindowsInstaller") == Some(1),
                estimated_size: registry::read_dword(&key, "EstimatedSize").map(f64::from),
                install_date: s("InstallDate"),
                reg_key: key,
            });
        }
    }
    out
}

/// Dead entries: the uninstaller and the install folder are gone. MSI entries are skipped - removing
/// them would leave the Windows Installer database inconsistent.
pub fn dead_entries(entries: &[UninstallEntry], exists: impl Fn(&str) -> bool) -> Vec<Leftover> {
    let mut out = Vec::new();
    for e in entries {
        if e.system_component || e.windows_installer || e.uninstall_string.to_lowercase().contains("msiexec") {
            continue;
        }
        let Some(exe) = command_path(&e.uninstall_string).filter(|p| !p.is_empty()) else { continue };
        if exists(&exe) {
            continue;
        }
        let loc = e.install_location.trim_matches(['"', ' ']);
        if !loc.is_empty() && exists(loc) {
            continue;
        }
        let (confidence, reason) = if loc.is_empty() {
            (Confidence::Medium, "Dead entry in the program list: the uninstaller is gone.")
        } else {
            (Confidence::High, "Dead entry in the program list: the uninstaller and the install folder are both gone.")
        };
        out.push(Leftover::new(LeftoverKind::Registry, e.reg_key.clone(), e.display_name.clone(), confidence, Text::new(reason)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_and_live_entries() {
        let e = |name: &str, un: &str, loc: &str| UninstallEntry { reg_key: format!("HKLM\\X\\{name}"), display_name: name.into(), uninstall_string: un.into(), install_location: loc.into(), ..Default::default() };
        let list = vec![
            e("Gone", "\"C:\\Gone\\unins000.exe\"", "C:\\Gone"),
            e("NoLoc", "C:\\Gone2\\u.exe /S", ""),
            e("Alive", "C:\\Here\\u.exe", ""),
            e("Msi", "MsiExec.exe /X{GUID}", ""),
        ];
        let dead = dead_entries(&list, |p| p.starts_with("C:\\Here"));
        assert_eq!(dead.len(), 2);
        assert_eq!(dead[0].confidence, Confidence::High);
        assert_eq!(dead[1].confidence, Confidence::Medium);
    }
}
