use std::path::PathBuf;
use std::sync::Mutex;

use crate::ctx::Ctx;
use crate::fs::{delete, measure};
use crate::scan::category::{Category, CleanOutcome, Kind, Measurement};
use crate::system::recycle;
use crate::text::Text;

/// Never selected in advance: emptying it is irreversible.
#[derive(Default)]
pub struct RecycleBin {
    measured: Mutex<f64>,
}

/// The account's Recycle Bin on each ticked drive: `X:\$Recycle.Bin\<SID>`.
fn folders(ctx: &Ctx) -> Vec<PathBuf> {
    ctx.drives.iter().map(|d| PathBuf::from(format!("{d}\\$Recycle.Bin\\{}", ctx.user.sid))).filter(|p| p.is_dir()).collect()
}

impl Category for RecycleBin {
    fn id(&self) -> &'static str {
        "recyclebin"
    }
    fn name(&self) -> Text {
        Text::new("Recycle Bin")
    }
    fn description(&self) -> Text {
        Text::new("Files you deleted. They cannot be restored after emptying.")
    }
    fn kind(&self) -> Kind {
        Kind::RecycleBin
    }

    fn measure(&self, ctx: &Ctx) -> Measurement {
        let m = measure::measure_paths(&folders(ctx), 0, ctx.excluded);
        if let Ok(mut b) = self.measured.lock() {
            *b = m.bytes;
        }
        Measurement { bytes: m.bytes, files: m.files, ..Default::default() }
    }

    fn clean(&self, ctx: &Ctx) -> CleanOutcome {
        let mut out = CleanOutcome { bytes: self.measured.lock().map(|b| *b).unwrap_or(0.0), ..Default::default() };
        if ctx.user.other_user {
            // The shell API would empty the administrator's bin; delete the starting account's folders
            for f in folders(ctx) {
                let r = delete::remove_folder_content(&f, 0, ctx.excluded);
                out.errors += r.errors;
                out.failures.extend(r.failures);
            }
        } else if let Err(e) = recycle::empty_recycle_bin() {
            out.failures.push(e);
        }
        out
    }
}
