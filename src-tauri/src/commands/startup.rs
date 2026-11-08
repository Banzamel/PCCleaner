use tauri::AppHandle;

use super::{blocking, lock, not_found, with_ctx};
use pccleaner_core::startup::{actions, entries, StartupEntry};
use pccleaner_core::{AppError, AppResult, Text};

fn refuse_in_preview(st: &crate::state::AppState) -> AppResult<()> {
    if st.cli.preview || lock(&st.config)?.preview {
        return Err(AppError::new(Text::new("Preview mode - startup apps are not changed."), "preview mode"));
    }
    Ok(())
}

#[tauri::command]
pub async fn startup_list(app: AppHandle) -> AppResult<Vec<StartupEntry>> {
    blocking(app, |app, st| {
        let sys = vec![st.user.system_drive.clone()];
        let list = with_ctx(app, st, sys, entries::list);
        *lock(&st.startup)? = list.clone();
        Ok(list)
    })
    .await
}

/// Turns a program on or off at sign-in, like Task Manager.
#[tauri::command]
pub async fn startup_set(app: AppHandle, id: String, enable: bool) -> AppResult<()> {
    blocking(app, move |_, st| {
        refuse_in_preview(st)?;
        let mut list = lock(&st.startup)?;
        let e = list.iter_mut().find(|e| e.id == id).ok_or_else(not_found)?;
        actions::set_enabled(e, enable)?;
        e.is_enabled = enable;
        st.log(&format!("Startup {}: {} ({}, {})", if enable { "enabled" } else { "disabled" }, e.name, e.source.english(), e.command));
        Ok(())
    })
    .await
}

/// Removes an entry permanently, with a backup. Returns where the backup is.
#[tauri::command]
pub async fn startup_remove(app: AppHandle, id: String) -> AppResult<Text> {
    blocking(app, move |_, st| {
        refuse_in_preview(st)?;
        let mut list = lock(&st.startup)?;
        let i = list.iter().position(|e| e.id == id).ok_or_else(not_found)?;
        let place = actions::remove(&list[i], &st.data)?;
        let e = list.remove(i);
        st.log(&format!("Startup entry removed: {} ({}, {}), backup: {}", e.name, e.source.english(), e.command, place.english()));
        Ok(place)
    })
    .await
}
