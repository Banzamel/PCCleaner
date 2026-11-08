use std::collections::HashSet;
use std::sync::Mutex;

use serde::Deserialize;

use crate::ctx::Ctx;
use crate::fs::walk;
use crate::scan::category::{Category, CleanOutcome, Kind, Measurement};
use crate::scan::driverstore::{self, DriverPackage};
use crate::system::{command, wmi};
use crate::text::Text;

/// Packages chosen by the last measurement; cleaning removes exactly those.
#[derive(Default)]
pub struct DriverStore {
    packages: Mutex<Vec<DriverPackage>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SignedDriver {
    inf_name: Option<String>,
}

fn find(ctx: &Ctx) -> Vec<DriverPackage> {
    let used: HashSet<String> = wmi::query::<SignedDriver>(wmi::CIMV2, "SELECT InfName FROM Win32_PnPSignedDriver")
        .into_iter()
        .filter_map(|d| d.inf_name.map(|n| n.to_lowercase()))
        .collect();
    let Ok(rd) = std::fs::read_dir(ctx.user.windir.join("INF")) else { return Vec::new() };
    let mut all = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_lowercase();
        if !(name.starts_with("oem") && name.ends_with(".inf")) {
            continue;
        }
        let Some(store_inf) = driverstore::store_location(&e.path()) else { continue };
        let Ok(bytes) = std::fs::read(&store_inf) else { continue };
        let v = driverstore::parse_inf(&driverstore::decode(&bytes));
        let folder = store_inf.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        // Plain sum: store files belong to the system, the lock test would treat them all as locked
        let size: u64 = walk::files(&folder, ctx.excluded).map(|f| f.len).sum();
        let original = store_inf.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        all.push(DriverPackage { inf: name, name: original, provider: v.provider, class: v.class, version: v.version, date: v.date, folder, bytes: size as f64 });
    }
    driverstore::removable(all, &used)
}

impl Category for DriverStore {
    fn id(&self) -> &'static str {
        "driverstore"
    }
    fn name(&self) -> Text {
        Text::new("Older driver versions (DriverStore)")
    }
    fn description(&self) -> Text {
        Text::new("Driver versions replaced by newer ones, e.g. old graphics drivers (often 1-5 GB). Drivers in use and the newest version of each are kept. Afterwards you cannot roll back to an older version in Device Manager.")
    }
    fn kind(&self) -> Kind {
        Kind::DriverStore
    }
    fn opt_in(&self) -> bool {
        true
    }

    fn measure(&self, ctx: &Ctx) -> Measurement {
        let list = find(ctx);
        let bytes = list.iter().map(|p| p.bytes).sum();
        let note = (!list.is_empty()).then(|| {
            let top = list.iter().take(3).map(|p| format!("{} {} ({})", p.name, p.version, p.provider)).collect::<Vec<_>>().join(", ");
            Text::new("{0} packages to remove, the largest: {1}.").arg(list.len()).arg(top)
        });
        if !list.is_empty() {
            ctx.out.log(&format!("driverstore: {}", list.iter().map(|p| format!("{}={} {}", p.inf, p.name, p.version)).collect::<Vec<_>>().join(", ")));
        }
        let files = list.len() as u64;
        if let Ok(mut p) = self.packages.lock() {
            *p = list;
        }
        Measurement { bytes, files, note, ..Default::default() }
    }

    fn clean(&self, ctx: &Ctx) -> CleanOutcome {
        let mut out = CleanOutcome::default();
        let list = self.packages.lock().map(|p| p.clone()).unwrap_or_default();
        for d in list {
            let r = command::run(&command::system32("pnputil.exe"), &["/delete-driver", &d.inf]);
            if r.ok() {
                out.bytes += d.bytes;
                ctx.out.log(&format!("Driver removed: {} {} {}", d.inf, d.name, d.version));
            } else {
                out.errors += 1;
                out.failures.push(format!("{} {} {}: {}", d.inf, d.name, d.version, r.stdout.trim()));
            }
        }
        out
    }
}
