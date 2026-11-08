//! Typed commands the window calls. Each one only wires a core function to the app state; heavy
//! work runs on a blocking thread with COM initialised.

pub mod analysis;
pub mod app;
pub mod assistant;
pub mod clean;
pub mod drivers;
pub mod scan;
pub mod settings;
pub mod startup;

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use tauri::{AppHandle, Manager};

use crate::reporter::EventReporter;
use crate::state::AppState;
use pccleaner_core::ctx::Ctx;
use pccleaner_core::fs::exclude::Excluded;
use pccleaner_core::system::ComGuard;
use pccleaner_core::{AppError, AppResult, Text};

/// Runs `f` on a blocking thread with COM initialised and the app state at hand.
pub async fn blocking<T, F>(app: AppHandle, f: F) -> AppResult<T>
where
    T: Send + 'static,
    F: FnOnce(&AppHandle, &AppState) -> AppResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let _com = ComGuard::new();
        let st = app.state::<AppState>();
        f(&app, &st)
    })
    .await
    .map_err(|e| AppError::raw(e.to_string()))?
}

pub fn lock<T>(m: &Mutex<T>) -> AppResult<MutexGuard<'_, T>> {
    m.lock().map_err(|_| AppError::raw("internal state is unavailable"))
}

pub fn reporter(app: &AppHandle, st: &AppState) -> EventReporter {
    EventReporter { app: app.clone(), logger: st.logger.clone() }
}

/// Excluded folders from Settings.
pub fn excluded(st: &AppState) -> Excluded {
    st.config.lock().map(|c| Excluded::new(&c.excluded_folders)).unwrap_or_default()
}

/// Runs `f` with a scan context for the given drives (the system drive is always included).
pub fn with_ctx<T>(app: &AppHandle, st: &AppState, drives: Vec<String>, f: impl FnOnce(&Ctx) -> T) -> T {
    let out = reporter(app, st);
    let ex = excluded(st);
    let ctx = Ctx::new(&st.user, &ex, &out, st.app_dir.clone(), drives);
    f(&ctx)
}

pub fn version(app: &AppHandle) -> String {
    app.package_info().version.to_string()
}

pub fn not_found() -> AppError {
    AppError::new(Text::new("The item is no longer on the list - scan again."), "unknown id")
}

pub fn exe() -> PathBuf {
    std::env::current_exe().unwrap_or_default()
}
