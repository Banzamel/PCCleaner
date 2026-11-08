use crate::ctx::Ctx;
use crate::scan::category::{Category, CleanOutcome, Kind, Measurement};
use crate::system::{command, drives};
use crate::text::Text;

/// WinSxS cleanup by DISM. Its gain cannot be known in advance; it is measured as free space after.
pub struct Dism;

impl Category for Dism {
    fn id(&self) -> &'static str {
        "dism"
    }
    fn name(&self) -> Text {
        Text::new("WinSxS component store (DISM)")
    }
    fn description(&self) -> Text {
        Text::new("Removes superseded Windows components, usually 1-3 GB. Takes a few minutes. Older updates can no longer be uninstalled afterwards.")
    }
    fn kind(&self) -> Kind {
        Kind::Dism
    }
    fn opt_in(&self) -> bool {
        true
    }
    fn measure(&self, _ctx: &Ctx) -> Measurement {
        Measurement::default()
    }
    fn clean(&self, ctx: &Ctx) -> CleanOutcome {
        let sys = &ctx.user.system_drive;
        let before = drives::free_space(sys, sys);
        let r = command::run(&command::system32("Dism.exe"), &["/Online", "/Cleanup-Image", "/StartComponentCleanup", "/ResetBase"]);
        if !r.ok() {
            ctx.out.log(&format!("DISM exit {}: {}", r.code, r.stdout.lines().last().unwrap_or_default()));
        }
        CleanOutcome { bytes: (drives::free_space(sys, sys) - before).max(0.0), errors: u64::from(!r.ok()), ..Default::default() }
    }
}
