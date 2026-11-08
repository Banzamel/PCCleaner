use std::path::Path;

use super::{approved, protection::protection, sources, EntryKind, StartupEntry};
use crate::ctx::Ctx;
use crate::scan::leftovers::missing::is_missing_on_disk;
use crate::scan::leftovers::names::command_path;
use crate::system::{registry, shortcut, tasks, version_info};
use crate::text::Text;

fn complete(ctx: &Ctx, mut e: StartupEntry, dead: Option<bool>) -> StartupEntry {
    let windir = ctx.user.windir.to_string_lossy().to_string();
    let file = e.file.clone().filter(|f| !f.is_empty());
    e.exists = file.as_deref().is_some_and(|f| Path::new(f).exists());
    // Dead the same way as in the plan: an empty MSI shortcut target or a path without a drive is not dead
    e.dead = dead.unwrap_or_else(|| file.as_deref().is_some_and(|f| is_missing_on_disk(f, &windir)));
    e.publisher = if e.exists { file.as_deref().and_then(|f| version_info::company_name(Path::new(f))).unwrap_or_default() } else { String::new() };
    e.protected = protection(&e.name, file.as_deref(), &windir);
    e
}

fn entry(kind: EntryKind, name: String, command: String, file: Option<String>, source: Text) -> StartupEntry {
    StartupEntry {
        id: String::new(), kind, name, command, file, source, exists: false, dead: false, publisher: String::new(), protected: None,
        is_enabled: true, reg_key: String::new(), approved_key: String::new(), approved_name: String::new(), path: String::new(),
    }
}

/// Programs started at sign-in: Run keys, Startup folders and Task Scheduler tasks with a sign-in or
/// boot trigger. Read-only.
pub fn list(ctx: &Ctx) -> Vec<StartupEntry> {
    let u = ctx.user;
    let mut out = Vec::new();
    for r in sources::run_keys(u) {
        for n in registry::value_names(&r.reg_key).into_iter().filter(|n| !n.is_empty()) {
            let command = registry::read_string(&r.reg_key, &n).unwrap_or_default();
            let mut e = entry(EntryKind::Run, n.clone(), command.clone(), command_path(&command), r.source.clone());
            e.is_enabled = approved::is_enabled(registry::read_binary(&r.approved_key, &n).as_deref());
            e.reg_key = r.reg_key.clone();
            e.approved_key = r.approved_key.clone();
            e.approved_name = n;
            out.push(complete(ctx, e, None));
        }
    }
    for f in sources::startup_folders(u) {
        let Ok(rd) = std::fs::read_dir(&f.folder) else { continue };
        for item in rd.flatten().filter(|i| i.path().is_file()) {
            let file_name = item.file_name().to_string_lossy().to_string();
            if file_name.eq_ignore_ascii_case("desktop.ini") {
                continue;
            }
            let path = item.path();
            let is_lnk = path.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk"));
            let target = if is_lnk { shortcut::target(&path) } else { Some(path.to_string_lossy().to_string()) };
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let mut e = entry(EntryKind::Folder, stem, path.to_string_lossy().to_string(), target, f.source.clone());
            e.is_enabled = approved::is_enabled(registry::read_binary(&f.approved_key, &file_name).as_deref());
            e.approved_key = f.approved_key.clone();
            e.approved_name = file_name;
            e.path = path.to_string_lossy().to_string();
            out.push(complete(ctx, e, None));
        }
    }
    let windir = u.windir.to_string_lossy().to_string();
    for t in tasks::list_third_party().into_iter().filter(|t| t.at_startup) {
        let progs: Vec<String> = t.actions.iter().filter_map(|(p, _)| command_path(p)).collect();
        let dead = !progs.is_empty() && progs.iter().all(|p| is_missing_on_disk(p, &windir));
        let command = t.actions.iter().map(|(p, a)| format!("{p} {a}").trim().to_string()).collect::<Vec<_>>().join("; ");
        let mut e = entry(EntryKind::Task, t.name.clone(), command, progs.first().cloned(), Text::new("Task Scheduler"));
        e.is_enabled = t.enabled;
        e.path = t.path.clone();
        out.push(complete(ctx, e, Some(dead)));
    }
    for (i, e) in out.iter_mut().enumerate() {
        e.id = format!("S{}", i + 1);
    }
    ctx.out.log(&format!("Startup: {} entries, {} enabled", out.len(), out.iter().filter(|e| e.is_enabled).count()));
    out
}
