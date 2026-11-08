use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

use super::names::is_skipped_name;
use crate::ctx::Ctx;
use crate::scan::categories::subfolders;
use crate::scan::leftover::{Confidence, Leftover, LeftoverKind};
use crate::system::registry;
use crate::text::Text;

static VDF_PATH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""path"\s+"([^"]+)""#).expect("valid regex"));

/// Library folders listed in libraryfolders.vdf (backslashes are doubled there).
pub fn parse_library_paths(vdf: &str) -> Vec<String> {
    VDF_PATH.captures_iter(vdf).map(|c| c[1].replace("\\\\", "\\")).collect()
}

/// A field of an appmanifest_*.acf file.
pub fn acf_field(text: &str, field: &str) -> Option<String> {
    let re = Regex::new(&format!(r#""{field}"\s+"([^"]+)""#)).ok()?;
    re.captures(text).map(|c| c[1].to_string())
}

pub fn libraries(ctx: &Ctx) -> Vec<PathBuf> {
    let Some(steam) = registry::read_string(&format!("{}\\Software\\Valve\\Steam", ctx.user.hkcu), "SteamPath") else { return Vec::new() };
    let steam = steam.replace('/', "\\");
    let mut libs = vec![steam.clone()];
    if let Ok(vdf) = std::fs::read_to_string(Path::new(&steam).join("steamapps\\libraryfolders.vdf")) {
        libs.extend(parse_library_paths(&vdf));
    }
    let mut seen = HashSet::new();
    libs.into_iter().filter(|l| seen.insert(l.to_lowercase())).map(PathBuf::from).collect()
}

fn manifests(lib: &Path) -> Vec<String> {
    let Ok(rd) = std::fs::read_dir(lib.join("steamapps")) else { return Vec::new() };
    rd.flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_lowercase();
            n.starts_with("appmanifest_") && n.ends_with(".acf")
        })
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .collect()
}

/// Names and folders of installed games, so their settings are not taken for leftovers.
pub fn installed_games(libs: &[PathBuf]) -> Vec<String> {
    libs.iter().flat_map(|l| manifests(l)).flat_map(|t| ["name", "installdir"].into_iter().filter_map(move |f| acf_field(&t, f))).collect()
}

/// Game folders without a manifest, shader caches and Workshop content of games that are gone.
pub fn orphaned(ctx: &Ctx, libs: &[PathBuf]) -> Vec<Leftover> {
    let mut out = Vec::new();
    for lib in libs.iter().filter(|l| ctx.on_selected_drive(&l.to_string_lossy())) {
        let sa = lib.join("steamapps");
        if !sa.is_dir() {
            continue;
        }
        let texts = manifests(lib);
        let dirs: HashSet<String> = texts.iter().filter_map(|t| acf_field(t, "installdir")).map(|d| d.to_lowercase()).collect();
        let ids: HashSet<String> = texts.iter().filter_map(|t| acf_field(t, "appid")).collect();
        for d in subfolders(&sa.join("common")) {
            let name = d.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if dirs.contains(&name.to_lowercase()) || is_skipped_name(&name) {
                continue;
            }
            let reason = Text::new("Steam game folder without a manifest - the game was uninstalled but its files remain.");
            let mut l = Leftover::new(LeftoverKind::Folder, d.to_string_lossy(), name, Confidence::High, reason);
            l.game = true;
            out.push(l);
        }
        for sub in ["shadercache", "workshop\\content"] {
            for d in subfolders(&sa.join(sub)) {
                let id = d.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) || ids.contains(&id) {
                    continue;
                }
                let reason = Text::new("Steam {0} for a game (appid {1}) that is no longer installed.").arg(sub).arg(&id);
                let mut l = Leftover::new(LeftoverKind::Folder, d.to_string_lossy(), format!("Steam {sub} {id}"), Confidence::High, reason);
                l.game = true;
                out.push(l);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vdf_and_acf() {
        let vdf = "\"libraryfolders\" { \"1\" { \"path\"\t\t\"G:\\\\SteamLibrary\" } }";
        assert_eq!(parse_library_paths(vdf), vec!["G:\\SteamLibrary"]);
        let acf = "\"AppState\" { \"appid\" \"457140\" \"name\" \"Oxygen Not Included\" \"installdir\" \"OxygenNotIncluded\" }";
        assert_eq!(acf_field(acf, "installdir").as_deref(), Some("OxygenNotIncluded"));
        assert_eq!(acf_field(acf, "appid").as_deref(), Some("457140"));
    }
}
