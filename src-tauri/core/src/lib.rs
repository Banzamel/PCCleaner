//! PC Cleaner logic without any user interface. The Tauri app (`src-tauri/src`) only wires these
//! functions to commands. Windows-only code sits behind `cfg(windows)` with empty fallbacks, so the
//! pure parts (rules, parsers, file walking) are tested with `cargo test` on any OS.

pub mod ai;
pub mod analysis;
pub mod autoclean;
pub mod clean;
pub mod cli;
pub mod ctx;
pub mod drivers;
pub mod error;
pub mod fs;
pub mod report;
pub mod scan;
pub mod startup;
pub mod storage;
pub mod system;
pub mod text;
pub mod user;

pub use error::{AppError, AppResult};
pub use text::{Arg, Text};
