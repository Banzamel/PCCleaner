use super::docker_cache::not_running;
use crate::ctx::Ctx;
use crate::report::Tone;
use crate::scan::category::{Category, CleanOutcome, Kind, Measurement};
use crate::scan::docker;
use crate::text::Text;

/// Everything in Docker, databases in volumes included - opt-in and clearly described.
pub struct DockerAll;

impl Category for DockerAll {
    fn id(&self) -> &'static str {
        "dockerall"
    }
    fn name(&self) -> Text {
        Text::new("Docker - everything (containers, images, volumes)")
    }
    fn description(&self) -> Text {
        Text::new("Stops and removes ALL containers, including running ones, then docker system prune -a --volumes: all images, volumes (INCLUDING DATABASES in them), networks and the build cache. Docker is left empty. Docker must be running. On Docker Desktop the space returns to the drive after the WSL disk is compacted.")
    }
    fn kind(&self) -> Kind {
        Kind::DockerAll
    }
    fn opt_in(&self) -> bool {
        true
    }

    fn measure(&self, _ctx: &Ctx) -> Measurement {
        let u = docker::usage();
        let note = if !u.running {
            Some(not_running())
        } else if u.in_use > 0.0 {
            Some(
                Text::new("Used: images {0}, containers {1}, volumes {2}, build cache {3}. Unused part: {4}.")
                    .size(u.used.images)
                    .size(u.used.containers)
                    .size(u.used.volumes)
                    .size(u.used.cache)
                    .size(u.reclaimable_total()),
            )
        } else {
            None
        };
        Measurement { bytes: u.in_use, note, ..Default::default() }
    }

    fn clean(&self, ctx: &Ctx) -> CleanOutcome {
        let before = docker::usage().in_use;
        let containers = docker::docker(&["ps", "-aq"]);
        if !containers.is_empty() {
            ctx.out.line(Text::new("Docker is stopping and removing containers: {0}...").arg(containers.len()), Tone::Gray);
            let mut args = vec!["rm", "-f", "-v"];
            args.extend(containers.iter().map(String::as_str));
            docker::docker(&args);
        }
        ctx.out.line(Text::new("Docker is removing images, volumes, networks and cache..."), Tone::Gray);
        docker::docker(&["system", "prune", "-a", "--volumes", "-f"]);
        // Since Docker 23 prune --volumes removes only anonymous volumes; named ones go separately
        let volumes = docker::docker(&["volume", "ls", "-q"]);
        if !volumes.is_empty() {
            ctx.out.line(Text::new("Docker is removing volumes: {0}...").arg(volumes.len()), Tone::Gray);
            let mut args = vec!["volume", "rm", "-f"];
            args.extend(volumes.iter().map(String::as_str));
            docker::docker(&args);
        }
        // Build history (the Builds list in Docker Desktop) of every builder; older buildx lacks it
        let mut builders: Vec<String> = docker::docker(&["buildx", "ls", "--format", "{{.Name}}"]).into_iter().filter(|b| !b.contains(char::is_whitespace)).collect();
        builders.dedup();
        if builders.is_empty() {
            docker::docker(&["buildx", "history", "rm", "--all"]);
        }
        for b in &builders {
            docker::docker(&["buildx", "history", "rm", "--all", "--builder", b]);
        }
        CleanOutcome { bytes: (before - docker::usage().in_use).max(0.0), ..Default::default() }
    }
}
