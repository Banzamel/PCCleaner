use std::collections::HashSet;
use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::fs::link;
use crate::fs::protect::Protected;
use crate::scan::categories::subfolders;
use crate::scan::leftover::{Confidence, Leftover, LeftoverKind};
use crate::text::Text;

/// Install folders of the games Epic Games Launcher knows, lower case.
pub fn installed(ctx: &Ctx) -> Vec<String> {
    let man = ctx.user.program_data.join("Epic\\EpicGamesLauncher\\Data\\Manifests");
    let Ok(rd) = std::fs::read_dir(man) else { return Vec::new() };
    let mut out = Vec::new();
    for e in rd.flatten().filter(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("item"))) {
        let parsed = std::fs::read_to_string(e.path()).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(t.trim_start_matches('\u{feff}')).ok());
        match parsed.as_ref().and_then(|v| v.get("InstallLocation")).and_then(|v| v.as_str()) {
            Some(loc) if !loc.is_empty() => out.push(loc.replace('/', "\\").trim_end_matches('\\').to_lowercase()),
            Some(_) => {}
            None => ctx.skip(&format!("Epic manifest {}", e.path().display()), "no InstallLocation"),
        }
    }
    out
}

/// Parent folder of a game folder; a drive stays as `g:\` (bare `g:` means the current folder).
fn parent(loc: &str) -> Option<String> {
    let p = &loc[..loc.rfind('\\')?];
    if p.is_empty() {
        None
    } else if p.len() == 2 && p.ends_with(':') {
        Some(format!("{p}\\"))
    } else {
        Some(p.to_string())
    }
}

/// An Epic game has an `.egstore` sub-folder. The game's parent folder (e.g. G:\ or Program Files) is NOT
/// an Epic library - it usually holds other programs and games - so a folder outside "Epic Games" is a
/// leftover only if it has `.egstore`, and a drive root is never treated as a library.
pub fn orphaned(ctx: &Ctx, installed: &[String], protected: &Protected) -> Vec<Leftover> {
    if installed.is_empty() && !ctx.user.program_data.join("Epic\\EpicGamesLauncher\\Data\\Manifests").is_dir() {
        return Vec::new();
    }
    let known: HashSet<&str> = installed.iter().map(String::as_str).collect();
    let mut parents: Vec<String> = vec![ctx.user.program_files.join("Epic Games").to_string_lossy().to_lowercase()];
    parents.extend(installed.iter().filter_map(|l| parent(l)));
    parents.sort();
    parents.dedup();
    let mut out = Vec::new();
    for r in parents.iter().filter(|r| ctx.on_selected_drive(r)) {
        let epic_folder = r.trim_end_matches('\\').rsplit('\\').next().is_some_and(|n| n.contains("epic"));
        for d in subfolders(&PathBuf::from(r)) {
            let name = d.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let full = d.to_string_lossy().to_string();
            if known.contains(full.to_lowercase().as_str()) || name == "Launcher" || name == "DirectXRedist" || name.starts_with("UE_") {
                continue;
            }
            let system = std::fs::symlink_metadata(&d).map(|m| link::attributes(&m) & 0x4 != 0).unwrap_or(true);
            if system || protected.is_protected(&full) {
                continue;
            }
            let egstore = d.join(".egstore").exists();
            if !egstore && !epic_folder {
                continue;
            }
            let conf = if egstore { Confidence::High } else { Confidence::Medium };
            let reason = Text::new("Epic game folder that is not on the Epic Games Launcher list of installed games.");
            let mut l = Leftover::new(LeftoverKind::Folder, full, name, conf, reason);
            l.game = true;
            out.push(l);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn parents() {
        assert_eq!(super::parent("g:\\fortnite").as_deref(), Some("g:\\"));
        assert_eq!(super::parent("c:\\program files\\epic games\\x").as_deref(), Some("c:\\program files\\epic games"));
    }
}
