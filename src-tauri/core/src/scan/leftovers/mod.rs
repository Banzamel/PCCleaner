//! Leftovers of uninstalled programs and games: folders, game files, dead entries in Apps & features,
//! vendor registry keys and dead startup entries.

pub mod dead_startup;
pub mod epic;
pub mod folders;
pub mod installed;
pub mod missing;
pub mod names;
pub mod steam;
pub mod uninstall;
pub mod vendor_keys;

use crate::ctx::Ctx;
use crate::fs::measure;
use crate::fs::protect::Protected;
use crate::scan::leftover::Leftover;

pub fn find(ctx: &Ctx) -> Vec<Leftover> {
    let protected = Protected::new(ctx.user, &ctx.app_dir);
    let entries = uninstall::read_all(&ctx.user.hkcu);
    let dead = uninstall::dead_entries(&entries, |p| std::path::Path::new(p).exists());
    let dead_keys: Vec<&str> = dead.iter().map(|d| d.path.as_str()).collect();
    let live: Vec<_> = entries.iter().filter(|e| !dead_keys.contains(&e.reg_key.as_str())).cloned().collect();
    let libs = steam::libraries(ctx);
    let installed = installed::collect(ctx, &live, &steam::installed_games(&libs));

    let (mut results, folder_names) = folders::find(ctx, &installed, &protected);

    let epic_installed = epic::installed(ctx);
    let mut games = steam::orphaned(ctx, &libs);
    games.extend(epic::orphaned(ctx, &epic_installed, &protected));
    // Safety net: a game folder must not contain an installed program, Steam library or Epic game
    let mut keep_paths: Vec<String> = libs.iter().map(|l| l.to_string_lossy().to_string()).collect();
    keep_paths.extend(epic_installed);
    for mut g in games {
        if protected.is_protected(&g.path) || installed.contains_installed(&g.path, &keep_paths) {
            ctx.out.log(&format!("Skipped protected game folder: {}", g.path));
            continue;
        }
        let m = measure::summarize_folder(std::path::Path::new(&g.path), ctx.excluded);
        g.bytes = m.bytes;
        g = g.with_date(m.last_write);
        results.push(g);
    }
    results.sort_by(|a, b| b.bytes.total_cmp(&a.bytes));

    let vendor = vendor_keys::find(ctx, &installed, &folder_names);
    let startup = dead_startup::find(ctx);
    results.extend(dead);
    results.extend(startup);
    results.extend(vendor);
    results
}
