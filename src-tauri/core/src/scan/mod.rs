//! The full scan: whitelisted categories, the user profile and leftovers of uninstalled programs.
//! It never deletes anything.

pub mod categories;
pub mod category;
pub mod docker;
pub mod driverstore;
pub mod folder;
pub mod leftover;
pub mod leftovers;
pub mod profile;
pub mod session;

use crate::ctx::Ctx;
use crate::report::Tone;
use crate::system::{drives, process};
use crate::text::Text;
use leftover::{Confidence, Leftover, LeftoverKind};
use session::{ScanSession, ScannedCategory};

/// Which parts the Settings page lets the scan cover.
#[derive(Debug, Clone, Default)]
pub struct ScanScope {
    pub skip_categories: Vec<String>,
    pub leftovers: bool,
    pub large_files: bool,
}

fn measure_categories(ctx: &Ctx, scope: &ScanScope) -> Vec<ScannedCategory> {
    let mut out = Vec::new();
    for cat in categories::all(ctx).into_iter().filter(|c| !scope.skip_categories.iter().any(|s| s == c.id())) {
        ctx.out.line(Text::new("{0}...").text(cat.name()), Tone::DarkGray);
        let m = cat.measure(ctx);
        let blocker = process::is_running(cat.blocking_processes()).then(|| Text::new("a browser is running - close it before cleaning"));
        out.push(ScannedCategory { cat, m, blocker });
    }
    out
}

/// Large files from Downloads and outside AppData: the user decides, never selected in advance.
fn large_files(profile: &profile::ProfileScan) -> Vec<Leftover> {
    profile
        .candidates
        .iter()
        .map(|c| {
            let name = c.path.rsplit(['\\', '/']).next().unwrap_or(&c.path).to_string();
            let reason = if c.in_downloads { "Large file in Downloads - an old installer or archive?" } else { "Large file (over 1 GB) in the user profile." };
            let mut l = Leftover::new(LeftoverKind::File, c.path.clone(), name, Confidence::Low, Text::new(reason)).with_date(c.modified);
            l.bytes = c.bytes;
            l.large_file = true;
            l
        })
        .collect()
}

pub fn full_scan(ctx: &Ctx, scope: &ScanScope) -> ScanSession {
    let sys = ctx.user.system_drive.clone();
    ctx.out.log(&format!("Drives: {}, user: {}{}", ctx.drives.join(", "), ctx.user.sid, if ctx.user.other_user { " (different account than administrator)" } else { "" }));
    let free_before = drives::free_space(&sys, &sys);
    let categories = measure_categories(ctx, scope);
    ctx.out.line(Text::new("Scanning the user profile (large files, AppData)..."), Tone::DarkGray);
    let profile = profile::scan(ctx);
    let mut leftovers = Vec::new();
    if scope.leftovers {
        ctx.out.line(Text::new("Looking for leftovers of uninstalled programs and games..."), Tone::DarkGray);
        leftovers = leftovers::find(ctx)
            .into_iter()
            .filter(|l| !l.goes_to_recycle_bin() || !(ctx.excluded.contains_str(&l.path) || ctx.excluded.holds_excluded(&l.path)))
            .collect();
    }
    if scope.large_files {
        leftovers.extend(large_files(&profile));
    }
    for (i, l) in leftovers.iter_mut().enumerate() {
        l.id = format!("R{}", i + 1);
    }
    let count = |k: &[LeftoverKind]| leftovers.iter().filter(|l| k.contains(&l.kind)).count();
    ctx.out.line(
        Text::new("Leftovers found: {0} folders, {1} files, {2} registry entries, {3} tasks.")
            .arg(count(&[LeftoverKind::Folder]))
            .arg(count(&[LeftoverKind::File]))
            .arg(count(&[LeftoverKind::Registry, LeftoverKind::Regvalue]))
            .arg(count(&[LeftoverKind::Task])),
        Tone::DarkGray,
    );
    if ctx.errors() > 0 {
        ctx.out.line(Text::new("Skipped items that could not be read: {0} (details in the log: {1})").arg(ctx.errors()).arg("PCCleaner_*.log"), Tone::Yellow);
    }
    ScanSession { categories, leftovers, profile, free_before, errors: ctx.errors(), drives: ctx.drives.clone() }
}
