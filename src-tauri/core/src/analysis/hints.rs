use std::sync::LazyLock;

use regex::Regex;

use crate::text::Text;

/// Known places with a fixed hint, so the most common big items are explained without the AI.
const PLACES: &[(&str, &str)] = &[
    (r"\\Windows\\WinSxS$", "Windows component store. Only DISM shrinks it (a plan category); the size is inflated by hard links."),
    (r"\\Windows\\Installer$", "MSI installer cache - do not delete by hand, it breaks uninstalling programs."),
    (r"\\Windows\\SoftwareDistribution$", "Windows updates - the \"Windows Update cache\" plan category."),
    (r"\\Windows\\System32\\DriverStore", "Driver store (e.g. the graphics driver). Do not delete by hand - the \"Older driver versions\" plan category removes old versions."),
    (r"\\Windows\\servicing", "Windows update servicing files - managed by the system."),
    (r"\\Windows\.old$", "Previous Windows version - a plan category."),
    (r"^[A-Z]:\\pagefile\.sys$", "Page file (virtual memory). Its size is set in the system settings."),
    (r"^[A-Z]:\\hiberfil\.sys$", "Hibernation and fast startup. \"powercfg /h off\" removes it but turns hibernation off."),
    (r"^[A-Z]:\\swapfile\.sys$", "Swap file for Store apps - managed by Windows."),
    (r"\\steamapps\\common\\[^\\]+$", "Steam game - uninstall it in Steam."),
    (r"\\steamapps\\(workshop|shadercache|downloading)", "Steam data (Workshop, shaders, downloads)."),
    (r"\\Epic Games\\[^\\]+$", "Epic Games game - uninstall it in the Epic launcher."),
    (r"\\swap\.vhdx$", "WSL swap file - removed by \"wsl --shutdown\"."),
    (r"\\AppData\\Local\\Temp(\\[^\\]+)?$", "Temporary files - the TEMP plan category (files in use are skipped)."),
    (r"(ext4|docker_data)\.vhdx$|DockerDesktopWSL|\\Docker\\wsl", "WSL/Docker disk. \"docker system prune\" and then compacting the VHDX frees space."),
    (r"\\AppData\\Local\\Packages\\[^\\]+$", "Microsoft Store app data - reset or uninstall it in Settings > Apps."),
    (r"\\node_modules$", "Node.js project dependencies - \"npm install\" restores them when needed."),
    (r"\\\.(gradle|m2|nuget|cargo|rustup)$", "Developer tool cache."),
    (r"\\(Downloads|Pobrane)$", "Downloaded files - often old installers and archives."),
    (r"\\PC ?Cleaner\\ai$", "The PC Cleaner AI engine and models."),
    (r"\.(iso|vhdx?|vmdk|qcow2)$", "Disk or virtual machine image."),
];

static COMPILED: LazyLock<Vec<(Regex, &'static str)>> =
    LazyLock::new(|| PLACES.iter().filter_map(|(p, t)| Regex::new(&format!("(?i){p}")).ok().map(|r| (r, *t))).collect());

pub fn hint(path: &str) -> Option<Text> {
    COMPILED.iter().find(|(r, _)| r.is_match(path)).map(|(_, t)| Text::new(t))
}

#[cfg(test)]
mod tests {
    use super::hint;

    #[test]
    fn known_places() {
        assert!(hint("C:\\Windows\\WinSxS").unwrap().key.starts_with("Windows component store"));
        assert!(hint("D:\\SteamLibrary\\steamapps\\common\\Game").unwrap().key.starts_with("Steam game"));
        assert!(hint("C:\\hiberfil.sys").is_some());
        assert!(hint("C:\\Users\\Jan\\Projects").is_none());
    }
}
