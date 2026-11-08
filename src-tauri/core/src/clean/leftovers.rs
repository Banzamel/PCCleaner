use std::path::{Path, PathBuf};

use super::backup;
use crate::ctx::Ctx;
use crate::fs::protect::Protected;
use crate::report::{StepState, Tone};
use crate::scan::leftover::{Leftover, LeftoverKind};
use crate::storage::paths::safe_name;
use crate::system::{recycle, registry, tasks};
use crate::text::Text;

/// Removes one leftover. Folders and files go to the Recycle Bin; registry entries and tasks are
/// deleted only after their backup was written.
fn remove(l: &Leftover, backups: &Path, protected: &Protected) -> Result<(), Text> {
    let file = |ext: &str| backups.join(format!("{}_{}.{ext}", l.id, safe_name(&l.name)));
    let os = |e: String| Text::new("{0}").arg(e);
    match l.kind {
        LeftoverKind::Folder => {
            // Last safeguard: never a drive root, the Recycle Bin or a system folder
            if protected.is_protected(&l.path) {
                return Err(Text::new("protected folder - kept"));
            }
            recycle::send_to_recycle_bin(Path::new(&l.path)).map_err(os)
        }
        LeftoverKind::File => recycle::send_to_recycle_bin(Path::new(&l.path)).map_err(os),
        LeftoverKind::Registry => {
            backup::export_key(&l.path, &file("reg"))?;
            registry::delete_tree(&l.path).map_err(|e| os(e.to_string()))
        }
        LeftoverKind::Regvalue => {
            let name = l.value_name.clone().unwrap_or_else(|| l.name.clone());
            backup::export_value(&l.path, &name, &file("reg"))?;
            registry::delete_value(&l.path, &name).map_err(|e| os(e.to_string()))
        }
        LeftoverKind::Task => {
            backup::export_task(&l.path, &file("xml"))?;
            tasks::delete(&l.path).map_err(os)
        }
    }
}

/// Description of a leftover for lists: tasks and registry values with their location.
pub fn display(l: &Leftover) -> Text {
    match l.kind {
        LeftoverKind::Task => Text::new("{0}: {1}").text(Text::new("Task Scheduler")).arg(&l.path),
        LeftoverKind::Registry | LeftoverKind::Regvalue if !l.path.ends_with(&format!("\\{}", l.name)) => Text::new("{0}  [{1}]").arg(&l.name).arg(&l.path),
        _ => Text::new("{0}").arg(&l.path),
    }
}

/// Cleans the selected leftovers: first everything that goes to the Recycle Bin, then registry
/// entries and tasks - two steps on the progress screen. Returns the bytes moved to the Recycle Bin.
pub fn clean(ctx: &Ctx, selected: &[&Leftover], backups: &Path) -> f64 {
    let protected = Protected::new(ctx.user, &ctx.app_dir);
    let mut to_bin = 0.0;
    for (step, bin) in [("leftovers-bin", true), ("leftovers-backup", false)] {
        let items: Vec<&&Leftover> = selected.iter().filter(|l| l.goes_to_recycle_bin() == bin).collect();
        if items.is_empty() {
            continue;
        }
        ctx.out.step(step, StepState::Running, 0.0, None);
        let (mut bytes, mut errors) = (0.0, 0);
        for l in items {
            match remove(l, backups, &protected) {
                Ok(()) => {
                    if bin {
                        bytes += l.bytes;
                        ctx.out.line(Text::new("{0} -> Recycle Bin").arg(&l.path), Tone::Green);
                    } else {
                        ctx.out.line(Text::new("{0}: deleted").text(display(l)), Tone::Green);
                    }
                    ctx.out.log(&format!("Removed ({:?}): {}", l.kind, l.path));
                }
                Err(e) => {
                    errors += 1;
                    ctx.out.line(Text::new("{0}: {1}").arg(&l.path).text(e.clone()), Tone::Yellow);
                    ctx.out.log(&format!("Leftover error {}: {}", l.path, e.english()));
                }
            }
        }
        let (state, note) = if errors > 0 { (StepState::Partial, Some(Text::new("failed: {0} - see the Log").arg(errors))) } else { (StepState::Ok, None) };
        ctx.out.step(step, state, bytes, note);
        to_bin += bytes;
    }
    if selected.iter().any(|l| matches!(l.kind, LeftoverKind::Registry | LeftoverKind::Regvalue)) {
        ctx.out.line(Text::new("Registry backups: {0} (double-click a .reg file to restore the key)").arg(backups.display()), Tone::DarkGray);
    }
    if selected.iter().any(|l| l.kind == LeftoverKind::Task) {
        ctx.out.line(Text::new("Task backups: {0} (Task Scheduler > Import Task restores a task from its .xml file)").arg(backups.display()), Tone::DarkGray);
    }
    to_bin
}

/// Number of backup files written in this run.
pub fn backup_count(dir: &PathBuf) -> usize {
    std::fs::read_dir(dir).map(|rd| rd.flatten().filter(|e| e.path().is_file()).count()).unwrap_or(0)
}
