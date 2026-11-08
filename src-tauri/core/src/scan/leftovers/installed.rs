use std::collections::HashSet;
use std::path::Path;

use super::names::{command_path, name_tokens, plain_name};
use super::uninstall::UninstallEntry;
use crate::ctx::Ctx;
use crate::fs::walk;
use crate::system::{process, registry, shortcut};

/// What is installed: name words, plain names and folders of programs, Store apps, services,
/// running processes, Start menu/desktop shortcuts and Steam games.
#[derive(Debug, Default)]
pub struct Installed {
    pub tokens: HashSet<String>,
    pub names: Vec<String>,
    pub paths: Vec<String>,
}

impl Installed {
    pub fn add_name(&mut self, text: &str) {
        self.tokens.extend(name_tokens(text));
        let n = plain_name(text);
        if n.len() >= 4 {
            self.names.push(n);
        }
    }

    /// A folder, or the folder of a file path.
    pub fn add_path(&mut self, p: &str) {
        let mut p = p.trim_matches(['"', ' ']).trim_end_matches('\\').to_string();
        let has_ext = p.rsplit_once('.').is_some_and(|(_, e)| (2..=4).contains(&e.len()) && e.chars().all(|c| c.is_alphanumeric() || c == '_') && !e.contains('\\'));
        if has_ext {
            p = p.rsplit_once('\\').map(|(d, _)| d.to_string()).unwrap_or_default();
        }
        if p.len() > 3 {
            self.paths.push(p.to_lowercase());
        }
    }

    /// The name or path matches an installed program.
    pub fn belongs(&self, name: &str, path: Option<&str>) -> bool {
        if name_tokens(name).iter().any(|t| self.tokens.contains(t)) {
            return true;
        }
        let n = plain_name(name);
        if n.len() >= 4 && self.names.iter().any(|z| z.contains(&n) || (z.len() >= 5 && n.contains(z.as_str()))) {
            return true;
        }
        if let Some(path) = path {
            let s = path.to_lowercase().trim_end_matches('\\').to_string();
            let (sd, ss) = (format!("{s}\\"), s.clone());
            return self.paths.iter().any(|p| *p == ss || p.starts_with(&sd) || ss.starts_with(&format!("{p}\\")));
        }
        false
    }

    /// The folder CONTAINS an installed program, Steam library or Epic game (e.g. G:\Steam, P:\Git).
    pub fn contains_installed(&self, path: &str, extra: &[String]) -> bool {
        let s = path.trim_end_matches('\\').to_lowercase();
        let sd = format!("{s}\\");
        self.paths.iter().chain(extra.iter()).map(|p| p.trim_end_matches('\\').to_lowercase()).any(|p| p == s || p.starts_with(&sd))
    }
}

/// Collects everything installed. `entries` are the live (not dead) uninstall entries.
pub fn collect(ctx: &Ctx, entries: &[UninstallEntry], steam_games: &[String]) -> Installed {
    let mut i = Installed::default();
    for e in entries {
        i.add_name(&e.display_name);
        i.add_name(&e.publisher);
        i.add_path(&e.install_location);
        for cmd in [&e.display_icon, &e.uninstall_string] {
            if let Some(p) = command_path(cmd) {
                i.add_path(&p);
            }
        }
    }
    let appx = format!("{}\\Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages", ctx.user.hkcu);
    for full in registry::subkeys(&appx) {
        i.add_name(full.split('_').next().unwrap_or(&full));
        if let Some(root) = registry::read_string(&format!("{appx}\\{full}"), "PackageRootFolder") {
            i.add_path(&root);
        }
    }
    for g in steam_games {
        i.add_name(g);
    }
    for p in process::list() {
        if let Some(path) = p.path {
            i.add_path(&path.to_string_lossy());
            i.add_name(path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default().as_str());
        }
    }
    let services = "HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services";
    for s in registry::subkeys(services) {
        if let Some(p) = registry::read_string(&format!("{services}\\{s}"), "ImagePath").and_then(|c| command_path(&c)) {
            i.add_path(&p);
        }
    }
    // Start menu and desktop shortcuts catch portable programs with no registry entry
    let u = ctx.user;
    for dir in [u.appdata.join("Microsoft\\Windows\\Start Menu"), u.program_data.join("Microsoft\\Windows\\Start Menu"), u.desktop(), u.public.join("Desktop")] {
        for f in walk::files(&dir, ctx.excluded).filter(|f| has_ext(&f.path, "lnk")) {
            i.add_name(&f.path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default());
            match shortcut::target(&f.path) {
                Some(t) if !t.is_empty() => i.add_path(&t),
                _ => {}
            }
        }
    }
    i
}

pub fn has_ext(p: &Path, ext: &str) -> bool {
    p.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installed() -> Installed {
        let mut i = Installed::default();
        i.add_name("Oxygen Not Included");
        i.add_path("\"C:\\Games\\Steam\\steam.exe\"");
        i.add_path("D:\\Tools\\Git\\");
        i
    }

    #[test]
    fn matches_by_name_and_path() {
        let i = installed();
        assert!(i.belongs("OxygenNotIncluded", None));
        assert!(i.belongs("Whatever", Some("C:\\Games\\Steam")));
        assert!(i.belongs("Whatever", Some("C:\\Games")), "a parent of an installed program");
        assert!(!i.belongs("OldThing", Some("C:\\Old")));
    }

    #[test]
    fn contains_installed_program() {
        let i = installed();
        assert!(i.contains_installed("D:\\Tools", &[]));
        assert!(!i.contains_installed("D:\\Tools\\Git\\sub", &[]));
        assert!(i.contains_installed("G:\\Lib", &["G:\\Lib\\steamapps".into()]));
    }
}
