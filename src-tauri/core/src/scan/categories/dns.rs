use crate::ctx::Ctx;
use crate::scan::category::{Category, CleanOutcome, Kind, Measurement};
use crate::system::command;
use crate::text::Text;

/// Frees no space; it is in the plan because it helps with network problems.
pub struct Dns;

impl Category for Dns {
    fn id(&self) -> &'static str {
        "dns"
    }
    fn name(&self) -> Text {
        Text::new("DNS cache")
    }
    fn description(&self) -> Text {
        Text::new("Remembered website addresses. Frees no space; helps with network problems.")
    }
    fn kind(&self) -> Kind {
        Kind::Dns
    }
    fn measure(&self, _ctx: &Ctx) -> Measurement {
        Measurement::default()
    }
    fn clean(&self, _ctx: &Ctx) -> CleanOutcome {
        let r = command::run(&command::system32("ipconfig.exe"), &["/flushdns"]);
        CleanOutcome { errors: u64::from(!r.ok()), ..Default::default() }
    }
}
