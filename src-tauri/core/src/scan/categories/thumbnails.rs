use std::path::PathBuf;
use std::time::Duration;

use crate::ctx::Ctx;
use crate::scan::category::{Category, CleanOutcome, Kind, Measurement};
use crate::system::process;
use crate::text::Text;

pub struct Thumbnails;

fn files(ctx: &Ctx) -> Vec<(PathBuf, u64)> {
    let dir = ctx.user.local_appdata.join("Microsoft\\Windows\\Explorer");
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    rd.flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            let md = e.metadata().ok()?;
            let ours = (name.starts_with("thumbcache_") || name.starts_with("iconcache_")) && name.ends_with(".db");
            (ours && md.is_file()).then(|| (e.path(), md.len()))
        })
        .collect()
}

impl Category for Thumbnails {
    fn id(&self) -> &'static str {
        "thumbnails"
    }
    fn name(&self) -> Text {
        Text::new("Thumbnails and icon cache")
    }
    fn description(&self) -> Text {
        Text::new("Picture and icon previews in File Explorer. Explorer restarts (the taskbar disappears for a few seconds).")
    }
    fn kind(&self) -> Kind {
        Kind::Thumbnails
    }

    fn measure(&self, ctx: &Ctx) -> Measurement {
        let f = files(ctx);
        Measurement { bytes: f.iter().map(|(_, l)| *l as f64).sum(), files: f.len() as u64, ..Default::default() }
    }

    /// Explorer keeps the databases open, so it is stopped for a moment.
    fn clean(&self, ctx: &Ctx) -> CleanOutcome {
        let mut out = CleanOutcome::default();
        process::kill("explorer");
        std::thread::sleep(Duration::from_secs(3));
        for (p, len) in files(ctx) {
            match std::fs::remove_file(&p) {
                Ok(()) => out.bytes += len as f64,
                Err(e) => {
                    out.errors += 1;
                    out.failures.push(format!("{}: {e}", p.display()));
                }
            }
        }
        // Windows restarts the shell itself; by hand only on our own account (it would run as the administrator)
        for _ in 0..10 {
            if process::is_running(&["explorer"]) {
                break;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        if !process::is_running(&["explorer"]) && !ctx.user.other_user {
            let _ = crate::system::command::hidden(&ctx.user.windir.join("explorer.exe").to_string_lossy()).spawn();
        }
        out
    }
}
