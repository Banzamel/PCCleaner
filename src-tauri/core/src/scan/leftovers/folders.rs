use std::collections::HashSet;
use std::path::PathBuf;

use super::installed::Installed;
use super::names::{is_skipped_name, plain_name};
use crate::ctx::Ctx;
use crate::fs::protect::Protected;
use crate::fs::{link, measure};
use crate::scan::categories::subfolders;
use crate::scan::leftover::{days_since, Confidence, Leftover, LeftoverKind};
use crate::text::Text;

const SYSTEM_ATTRIBUTE: u32 = 0x4;

/// Program folders in AppData, LocalLow, ProgramData and Program Files (plus game saves) that match
/// no installed program. Returns the leftovers and the plain names of every folder seen (registry
/// keys of the same name are then not offered).
pub fn find(ctx: &Ctx, installed: &Installed, protected: &Protected) -> (Vec<Leftover>, HashSet<String>) {
    let u = ctx.user;
    let roots: Vec<(PathBuf, bool)> = vec![
        (u.appdata.clone(), false),
        (u.local_appdata.clone(), false),
        (u.local_appdata.join("Programs"), false),
        (u.local_low(), false),
        (u.program_data.clone(), false),
        (u.program_files.clone(), false),
        (u.program_files_x86.clone(), false),
        (u.documents().join("My Games"), true),
        (u.saved_games(), true),
    ];
    let mut all_names = HashSet::new();
    let mut to_check = Vec::new();
    for (root, saves) in &roots {
        for d in subfolders(root) {
            let name = d.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            all_names.insert(plain_name(&name));
            let Ok(md) = std::fs::symlink_metadata(&d) else { continue };
            if link::is_link(&md) || link::attributes(&md) & SYSTEM_ATTRIBUTE != 0 {
                continue;
            }
            let full = d.to_string_lossy().to_string();
            if is_skipped_name(&name) || protected.is_protected(&full) || installed.belongs(&name, Some(&full)) {
                continue;
            }
            to_check.push((d, name, *saves));
        }
    }
    let mut out = Vec::new();
    for (d, name, saves) in to_check {
        ctx.out.status(Text::new("{0}").arg(d.display()));
        let m = measure::summarize_folder(&d, ctx.excluded);
        let days = days_since(m.last_write);
        if days < 30.0 {
            continue; // something still writes there
        }
        // An uninstalled program leaves settings and logs, not .exe files - a folder with .exe files
        // may be a portable program with no registry entry, so it is not selected in advance
        let (confidence, reason) = if saves {
            (Confidence::Low, "Game saves/settings - you may want to keep them.")
        } else if m.has_exe {
            (Confidence::Low, "No installed program with this name, but the folder contains .exe files - it may be a portable program.")
        } else if days >= 180.0 {
            (Confidence::High, "No installed program with this name.")
        } else {
            (Confidence::Medium, "No installed program with this name.")
        };
        let mut l = Leftover::new(LeftoverKind::Folder, d.to_string_lossy(), name, confidence, Text::new(reason)).with_date(m.last_write);
        l.bytes = m.bytes;
        out.push(l);
    }
    (out, all_names)
}
