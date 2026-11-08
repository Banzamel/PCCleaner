use std::collections::HashSet;
use std::path::PathBuf;

use super::installed::has_ext;
use super::missing::is_missing_on_disk;
use super::names::command_path;
use crate::ctx::Ctx;
use crate::fs::walk;
use crate::scan::leftover::{Confidence, Leftover, LeftoverKind};
use crate::startup::sources;
use crate::system::{registry, shortcut, tasks};
use crate::text::Text;

/// Startup entries, shortcuts and scheduled tasks that point to a program that no longer exists.
pub fn find(ctx: &Ctx) -> Vec<Leftover> {
    let u = ctx.user;
    let windir = u.windir.to_string_lossy().to_string();
    let missing = |p: &str| is_missing_on_disk(p, &windir);
    let mut out = Vec::new();
    for r in sources::run_keys(u) {
        for n in registry::value_names(&r.reg_key).into_iter().filter(|n| !n.is_empty()) {
            let Some(target) = registry::read_string(&r.reg_key, &n).and_then(|c| command_path(&c)) else { continue };
            if missing(&target) {
                let reason = Text::new("Dead startup entry: the program {0} no longer exists.").arg(&target);
                let mut l = Leftover::new(LeftoverKind::Regvalue, r.reg_key.clone(), n.clone(), Confidence::High, reason);
                l.value_name = Some(n);
                out.push(l);
            }
        }
    }
    // Shortcuts: certain in the Startup folders and the Start menu, medium on the desktop (someone may have made them)
    let mut places: Vec<(PathBuf, Confidence, bool)> = sources::startup_folders(u).into_iter().map(|f| (f.folder, Confidence::High, false)).collect();
    places.push((u.appdata.join("Microsoft\\Windows\\Start Menu\\Programs"), Confidence::High, true));
    places.push((u.program_data.join("Microsoft\\Windows\\Start Menu\\Programs"), Confidence::High, true));
    places.push((u.desktop(), Confidence::Medium, false));
    places.push((u.public.join("Desktop"), Confidence::Medium, false));
    let mut seen = HashSet::new();
    for (dir, conf, recurse) in places {
        let files: Vec<walk::FileEntry> = if recurse {
            walk::files(&dir, ctx.excluded).collect()
        } else {
            std::fs::read_dir(&dir).map(|rd| rd.flatten().filter_map(|e| entry(e.path())).collect()).unwrap_or_default()
        };
        for f in files.into_iter().filter(|f| has_ext(&f.path, "lnk")) {
            if !seen.insert(f.path.clone()) {
                continue;
            }
            // Empty for MSI shortcuts and Store apps
            let Some(target) = shortcut::target(&f.path) else { continue };
            if missing(&target) {
                let name = f.path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                let reason = Text::new("Shortcut to a program that no longer exists ({0}).").arg(&target);
                let mut l = Leftover::new(LeftoverKind::File, f.path.to_string_lossy(), name, conf, reason).with_date(f.modified);
                l.bytes = f.len as f64;
                out.push(l);
            }
        }
    }
    // Tasks: dead when none of their programs exist
    for t in tasks::list_third_party() {
        let progs: Vec<String> = t.actions.iter().filter_map(|(p, _)| command_path(p)).collect();
        if progs.is_empty() || !progs.iter().all(|p| missing(p)) {
            continue;
        }
        let reason = Text::new("Scheduled task runs a program that no longer exists ({0}).").arg(progs.join(", "));
        let mut l = Leftover::new(LeftoverKind::Task, t.path.clone(), t.name.clone(), Confidence::High, reason);
        l.folder = Some(t.folder.clone());
        out.push(l);
    }
    out
}

fn entry(path: PathBuf) -> Option<walk::FileEntry> {
    let md = std::fs::symlink_metadata(&path).ok()?;
    md.is_file().then(|| walk::FileEntry { len: md.len(), modified: md.modified().ok(), attrs: crate::fs::link::attributes(&md), path })
}
