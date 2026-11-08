//! Cleaning what the user confirmed. Items are chosen by id from the scan session, so the window can
//! only pick from what the scan found - it cannot add a path.

pub mod backup;
pub mod leftovers;

use std::time::Instant;

use serde::Serialize;

use crate::ctx::Ctx;
use crate::report::{StepState, Tone};
use crate::scan::leftover::Leftover;
use crate::scan::session::{ScanSession, ScannedCategory};
use crate::storage::history::{self, HistoryEntry};
use crate::storage::paths::{stamp, DataPaths};
use crate::system::{drives, process, restore_point};
use crate::text::{format_size, Text};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub total: f64,
    pub duration_ms: u64,
    pub free_before: f64,
    pub free_after: f64,
    pub backups: Option<String>,
    pub backup_count: usize,
    pub restore_point: Option<Text>,
    /// Bytes in files that were in use (not deleted).
    pub locked: f64,
    /// Bytes moved to the Recycle Bin (freed only when it is emptied).
    pub in_recycle_bin: f64,
}

/// A restore point is made before changes the Recycle Bin cannot undo: registry, tasks, drivers.
pub fn restore_point_needed(cats: &[&ScannedCategory], lefts: &[&Leftover]) -> bool {
    lefts.iter().any(|l| !l.goes_to_recycle_bin()) || cats.iter().any(|c| c.cat.id() == "driverstore")
}

fn clean_category(ctx: &Ctx, c: &ScannedCategory) -> f64 {
    let id = c.cat.id();
    ctx.out.step(id, StepState::Running, 0.0, None);
    ctx.out.status(Text::new("{0}...").text(c.cat.name()));
    if process::is_running(c.cat.blocking_processes()) {
        ctx.out.line(Text::new("{0}: {1}").text(c.cat.name()).text(Text::new("skipped (a browser is running)")), Tone::Yellow);
        ctx.out.log(&format!("{id}: skipped - a browser is running"));
        ctx.out.step(id, StepState::Skipped, 0.0, Some(Text::new("skipped: a browser is running")));
        return 0.0;
    }
    let r = c.cat.clean(ctx);
    for f in &r.failures {
        ctx.out.log(&format!("{id}: not deleted {f}"));
    }
    let mut notes = Vec::new();
    if r.errors > 0 {
        notes.push(Text::new("{0} in use - skipped").arg(r.errors));
    }
    if !r.skipped_apps.is_empty() {
        notes.push(Text::new("skipped: {0} is running").arg(r.skipped_apps.join(", ")));
        ctx.out.log(&format!("{id}: skipped running {}", r.skipped_apps.join(", ")));
    }
    ctx.out.log(&format!("{id}: {} freed, {} skipped", format_size(r.bytes), r.errors));
    let tone = if r.bytes > 50.0 * 1024.0 * 1024.0 { Tone::Green } else { Tone::Gray };
    ctx.out.line(Text::new("{0}: {1}").text(c.cat.name()).size(r.bytes), tone);
    let state = if !r.skipped_apps.is_empty() && r.bytes == 0.0 {
        StepState::Skipped
    } else if notes.is_empty() {
        StepState::Ok
    } else {
        StepState::Partial
    };
    let note = match notes.len() {
        0 => None,
        1 => notes.pop(),
        _ => Some(Text::new("{0}; {1}").text(notes.remove(0)).text(notes.remove(0))),
    };
    ctx.out.step(id, state, r.bytes, note);
    r.bytes
}

/// Cleans the selected categories and leftovers and records the run in history.json.
pub fn run(ctx: &Ctx, session: &ScanSession, category_ids: &[String], leftover_ids: &[String], data: &DataPaths, version: &str) -> CleanResult {
    let started = Instant::now();
    let cats: Vec<&ScannedCategory> = session.categories.iter().filter(|c| category_ids.iter().any(|i| i == c.cat.id())).collect();
    let lefts: Vec<&Leftover> = session.leftovers.iter().filter(|l| leftover_ids.contains(&l.id)).collect();
    ctx.out.log(&format!("Plan: categories [{}], leftovers [{}]", cats.iter().map(|c| c.cat.id()).collect::<Vec<_>>().join(", "), leftover_ids.join(", ")));

    let mut restore = None;
    if restore_point_needed(&cats, &lefts) {
        ctx.out.step("restorepoint", StepState::Running, 0.0, None);
        ctx.out.line(Text::new("Creating a System Restore point (up to 30 s)..."), Tone::Gray);
        let r = restore_point::create(&format!("PC Cleaner {version}"));
        ctx.out.line(Text::new("Restore point: {0}").text(r.description.clone()), if r.ok { Tone::Green } else { Tone::Yellow });
        ctx.out.log(&format!("Restore point: {}", r.description.english()));
        ctx.out.step("restorepoint", if r.ok { StepState::Ok } else { StepState::Partial }, 0.0, Some(r.description.clone()));
        restore = Some(r.description);
    }

    let total: f64 = cats.iter().map(|c| clean_category(ctx, c)).sum();
    let locked = cats.iter().map(|c| c.m.locked).sum();

    let backups_dir = data.backups(&stamp());
    let mut in_bin = 0.0;
    if !lefts.is_empty() {
        ctx.out.line(Text::new("Leftovers of uninstalled programs and games:"), Tone::Cyan);
        in_bin = leftovers::clean(ctx, &lefts, &backups_dir);
    }
    let backup_count = leftovers::backup_count(&backups_dir);

    let sys = &ctx.user.system_drive;
    let free_after = drives::free_space(sys, sys);
    ctx.out.log(&format!("Done. Removed {}, free on {sys}: {} -> {}", format_size(total), format_size(session.free_before), format_size(free_after)));
    let entry = HistoryEntry { date: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(), drive: sys.clone(), total, free_before: session.free_before, free_after };
    if let Err(e) = history::add(data, entry) {
        ctx.out.log(&format!("History: {e}"));
    }
    CleanResult {
        total,
        duration_ms: started.elapsed().as_millis() as u64,
        free_before: session.free_before,
        free_after,
        backups: (backup_count > 0).then(|| backups_dir.to_string_lossy().to_string()),
        backup_count,
        restore_point: restore,
        locked,
        in_recycle_bin: in_bin,
    }
}
