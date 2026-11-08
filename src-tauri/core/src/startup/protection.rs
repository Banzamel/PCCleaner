use crate::scan::leftovers::names::like;
use crate::text::Text;

/// Security software, drivers and hardware panels: turning them off can break the touchpad, the sound
/// or the protection.
const PROTECTED: &[&str] = &[
    "securityhealth*", "*defender*", "msmpeng*", "*antivirus*", "avast*", "avg*", "egui*", "eset*", "avp*", "kaspersky*", "bdagent*",
    "bitdefender*", "norton*", "mcafee*", "mbam*", "malwarebytes*", "sophos*", "rthdvcpl*", "rtkaud*", "rtkngui*", "realtek*", "igfx*",
    "synaptics*", "syntp*", "etdctrl*", "elan*", "waves*", "maxxaudio*", "dolby*",
];

/// Why an entry is read-only, or `None` when the user may change it.
pub fn protection(name: &str, file: Option<&str>, windir: &str) -> Option<Text> {
    let win = format!("{}\\", windir.trim_end_matches('\\').to_lowercase());
    if file.is_some_and(|f| f.to_lowercase().starts_with(&win)) {
        return Some(Text::new("part of Windows"));
    }
    let stem = file.map(|f| {
        let leaf = f.rsplit(['\\', '/']).next().unwrap_or(f);
        leaf.rsplit_once('.').map(|(s, _)| s).unwrap_or(leaf).to_string()
    });
    let candidates = [Some(name.to_string()), stem].into_iter().flatten().filter(|s| !s.is_empty());
    for x in candidates {
        if PROTECTED.iter().any(|p| like(&x, p)) {
            return Some(Text::new("security software, a driver or a hardware panel - turning it off may break something"));
        }
    }
    if name.to_lowercase().contains("update") {
        return Some(Text::new("program updates - without them it gets no security fixes"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::protection;

    #[test]
    fn rules() {
        let w = "C:\\Windows";
        assert_eq!(protection("SecurityHealth", Some("C:\\Windows\\system32\\SecurityHealthSystray.exe"), w).unwrap().key, "part of Windows");
        assert!(protection("Realtek HD Audio", Some("C:\\Program Files\\Realtek\\RtkNGUI64.exe"), w).is_some());
        assert!(protection("GoogleUpdate", Some("C:\\X\\g.exe"), w).unwrap().key.starts_with("program updates"));
        assert!(protection("Discord", Some("C:\\Users\\Jan\\AppData\\Local\\Discord\\Update.exe"), w).is_none());
    }
}
