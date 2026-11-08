use serde::Serialize;

use crate::ctx::Ctx;
use crate::text::Text;

/// How a category is measured and cleaned; the UI uses it for sizes, notes and warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Folder,
    RecycleBin,
    Thumbnails,
    Dns,
    WinOld,
    Dism,
    DriverStore,
    Docker,
    DockerAll,
}

/// What a measurement found.
#[derive(Debug, Clone, Default)]
pub struct Measurement {
    pub bytes: f64,
    pub files: u64,
    /// Bytes in files that are in use and will not be deleted.
    pub locked: f64,
    pub note: Option<Text>,
}

/// What a cleanup of one category did.
#[derive(Debug, Clone, Default)]
pub struct CleanOutcome {
    pub bytes: f64,
    /// Items that could not be deleted (in use, no permission).
    pub errors: u64,
    /// Apps whose cache was skipped because they were running.
    pub skipped_apps: Vec<String>,
    /// Per-item failures for the log.
    pub failures: Vec<String>,
}

/// One whitelisted junk category. Each one lives in its own file in `categories/`.
pub trait Category: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> Text;
    fn description(&self) -> Text;
    fn kind(&self) -> Kind;
    /// Never selected in advance (irreversible or slow).
    fn opt_in(&self) -> bool {
        false
    }
    /// Programs that must not run while the whole category is cleaned (browsers).
    fn blocking_processes(&self) -> &[&'static str] {
        &[]
    }
    fn measure(&self, ctx: &Ctx) -> Measurement;
    fn clean(&self, ctx: &Ctx) -> CleanOutcome;
}

/// Size that means something for this kind (DNS and DISM have none; Docker only when it runs).
pub fn measurable(kind: Kind, bytes: f64) -> bool {
    match kind {
        Kind::Folder | Kind::RecycleBin | Kind::Thumbnails | Kind::WinOld | Kind::DriverStore => true,
        Kind::Docker | Kind::DockerAll => bytes > 0.0,
        Kind::Dns | Kind::Dism => false,
    }
}
