use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::{blocking, lock, version};
use pccleaner_core::drivers::{wu, DriverCheck};
use pccleaner_core::system::restore_point;
use pccleaner_core::{AppError, AppResult, Text};

#[derive(Serialize, Clone)]
struct DriverState<'a> {
    id: &'a str,
    state: &'static str,
    error: String,
}

/// Looks for newer drivers on Windows Update (1-3 minutes). Only on request.
#[tauri::command]
pub async fn drivers_check(app: AppHandle) -> AppResult<DriverCheck> {
    blocking(app, |_, st| {
        st.log("Drivers: searching Windows Update");
        let r = wu::find().map_err(|e| AppError::new(Text::new("Could not check Windows Update: {0}. Check the internet connection and that the Windows Update service is running.").arg(&e), e))?;
        st.log(&format!("Drivers: {}", r.drivers.iter().map(|d| format!("{} [{}]", d.title, d.current_version.clone().unwrap_or_default())).collect::<Vec<_>>().join(", ")));
        st.log(&format!("Devices with a problem: {}", r.problems.iter().map(|p| format!("{} ({})", p.name, p.code)).collect::<Vec<_>>().join(", ")));
        Ok(r)
    })
    .await
}

/// Creates a restore point, then installs the drivers one by one; each state goes to the window
/// as `pcc://driver`. Returns the restore point result.
#[tauri::command]
pub async fn drivers_install(app: AppHandle, ids: Vec<String>) -> AppResult<Text> {
    blocking(app, move |app, st| {
        if st.cli.preview || lock(&st.config)?.preview {
            return Err(AppError::new(Text::new("Preview mode - installing is disabled."), "preview mode"));
        }
        st.installing_drivers.store(true, Ordering::SeqCst);
        let restore = restore_point::create(&format!("PC Cleaner {}", version(app)));
        st.log(&format!("Restore point (drivers): {}", restore.description.english()));
        wu::install(&ids, &|id, state| {
            let error = match &state {
                wu::InstallState::Fail(e) => e.clone(),
                _ => String::new(),
            };
            st.log(&format!("Driver {id}: {} {error}", state.code()));
            let _ = app.emit("pcc://driver", DriverState { id, state: state.code(), error });
        });
        st.installing_drivers.store(false, Ordering::SeqCst);
        Ok(restore.description)
    })
    .await
}
