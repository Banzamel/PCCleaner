use crate::ctx::Ctx;
use crate::scan::category::{Category, CleanOutcome, Kind, Measurement};
use crate::scan::docker;
use crate::text::Text;

/// Build cache only; images, containers and volumes (databases) stay.
pub struct DockerCache;

pub fn not_running() -> Text {
    Text::new("Docker is not running - start it to measure and clean.")
}

impl Category for DockerCache {
    fn id(&self) -> &'static str {
        "docker"
    }
    fn name(&self) -> Text {
        Text::new("Docker build cache")
    }
    fn description(&self) -> Text {
        Text::new("Build cache only. Images, containers and volumes (including databases) are kept.")
    }
    fn kind(&self) -> Kind {
        Kind::Docker
    }
    fn opt_in(&self) -> bool {
        true
    }

    fn measure(&self, _ctx: &Ctx) -> Measurement {
        let u = docker::usage();
        Measurement { bytes: u.reclaimable.cache, note: (!u.running).then(not_running), ..Default::default() }
    }

    fn clean(&self, ctx: &Ctx) -> CleanOutcome {
        let before = docker::usage().reclaimable.cache;
        if let Some(last) = docker::docker(&["builder", "prune", "-a", "-f"]).last() {
            ctx.out.log(&format!("Docker: {last}"));
        }
        CleanOutcome { bytes: (before - docker::usage().reclaimable.cache).max(0.0), ..Default::default() }
    }
}
