use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::fs::measure;
use crate::scan::category::{Category, CleanOutcome, Kind, Measurement};
use crate::system::{command, drives, registry};
use crate::text::Text;

/// Removed by Windows Disk Cleanup (`cleanmgr /sagerun`), like "Clean up system files". Doing it by
/// hand is dangerous: junctions in Windows.old point to the current C:\Users.
pub struct WindowsOld;

const SAGE_ID: &str = "StateFlags0777";
const SAGE_KEY: &str = "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VolumeCaches";
const HANDLERS: [&str; 2] = ["Previous Installations", "Temporary Setup Files"];

fn paths(ctx: &Ctx) -> Vec<PathBuf> {
    ["Windows.old", "$WINDOWS.~BT", "$WINDOWS.~WS"].iter().map(|p| ctx.user.sys_path(p)).collect()
}

impl Category for WindowsOld {
    fn id(&self) -> &'static str {
        "winold"
    }
    fn name(&self) -> Text {
        Text::new("Previous Windows version (Windows.old)")
    }
    fn description(&self) -> Text {
        Text::new("A copy of the system from before a major update, usually 10-30 GB. Afterwards you cannot go back to the previous Windows version.")
    }
    fn kind(&self) -> Kind {
        Kind::WinOld
    }
    fn opt_in(&self) -> bool {
        true
    }

    fn measure(&self, ctx: &Ctx) -> Measurement {
        let m = measure::measure_paths(&paths(ctx), 0, ctx.excluded);
        Measurement { bytes: m.bytes, files: m.files, ..Default::default() }
    }

    fn clean(&self, ctx: &Ctx) -> CleanOutcome {
        let sys = &ctx.user.system_drive;
        let before = drives::free_space(sys, sys);
        for h in HANDLERS {
            let key = format!("{SAGE_KEY}\\{h}");
            if registry::key_exists(&key) {
                let _ = registry::set_dword(&key, SAGE_ID, 2);
            }
        }
        command::run(&command::system32("cleanmgr.exe"), &["/sagerun:777"]);
        for h in HANDLERS {
            let _ = registry::delete_value(&format!("{SAGE_KEY}\\{h}"), SAGE_ID);
        }
        let errors = paths(ctx).iter().filter(|p| p.exists()).count() as u64;
        CleanOutcome { bytes: (drives::free_space(sys, sys) - before).max(0.0), errors, ..Default::default() }
    }
}
