use tauri::AppHandle;

use super::{blocking, exe, lock};
use pccleaner_core::autoclean;
use pccleaner_core::storage::config::Config;
use pccleaner_core::storage::logger;
use pccleaner_core::{AppError, AppResult, Text};

#[tauri::command]
pub async fn get_config(app: AppHandle) -> AppResult<Config> {
    blocking(app, |_, st| Ok(lock(&st.config)?.clone())).await
}

/// Saves settings.json. The automatic cleaning task changes at once, not after a restart; its error,
/// if any, is returned for the window to show.
#[tauri::command]
pub async fn save_config(app: AppHandle, config: Config) -> AppResult<Option<Text>> {
    blocking(app, move |_, st| {
        config.save(&st.data).map_err(|e| AppError::new(Text::new("Could not save the settings: {0}").arg(&e), e.to_string()))?;
        st.log(&format!("Settings saved: {}", config.to_value()));
        let mode = config.auto_clean.clone();
        *lock(&st.config)? = config;
        // The task runs as the signed-in account - with another administrator's password it would
        // clean the wrong profile, so it is not set up then
        if st.user.other_user {
            return Ok(None);
        }
        Ok(autoclean::apply(&mode, &exe()).err().map(|e| {
            st.log(&format!("Automatic cleaning task: {e}"));
            Text::new("Could not set up the automatic cleaning: {0}").arg(e)
        }))
    })
    .await
}

/// Marks today's automatic cleaning as done.
#[tauri::command]
pub async fn auto_clean_done(app: AppHandle) -> AppResult<()> {
    blocking(app, |_, st| {
        let mut c = lock(&st.config)?;
        c.auto_clean_last = chrono::Local::now().format("%Y-%m-%d").to_string();
        c.save(&st.data)?;
        Ok(())
    })
    .await
}

/// Deletes the old run logs; the current one and the registry backups stay.
#[tauri::command]
pub async fn clear_logs(app: AppHandle) -> AppResult<usize> {
    blocking(app, |_, st| {
        let n = logger::remove_old(&st.data.logs(), &st.logger.path, 0);
        st.log(&format!("Deleted {n} logs from Settings."));
        Ok(n)
    })
    .await
}
