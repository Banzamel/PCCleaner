use std::sync::atomic::Ordering;

use tauri::AppHandle;

use super::{blocking, lock, version, with_ctx};
use pccleaner_core::clean::{self, CleanResult};
use pccleaner_core::{AppError, AppResult, Text};

/// Cleans the categories and leftovers the user confirmed, chosen by id from the last scan.
#[tauri::command]
pub async fn clean(app: AppHandle, categories: Vec<String>, leftovers: Vec<String>) -> AppResult<CleanResult> {
    blocking(app, move |app, st| {
        if st.cli.preview || lock(&st.config)?.preview {
            return Err(AppError::new(Text::new("Preview mode - cleaning disabled"), "preview mode"));
        }
        // The assistant is no longer needed - frees VRAM and RAM for the cleanup
        lock(&st.ollama)?.stop();
        st.cleaning.store(true, Ordering::SeqCst);
        let result = (|| {
            let guard = lock(&st.session)?;
            let session = guard.as_ref().ok_or_else(super::not_found)?;
            let drives = session.drives.clone();
            Ok(with_ctx(app, st, drives, |ctx| clean::run(ctx, session, &categories, &leftovers, &st.data, &version(app))))
        })();
        st.cleaning.store(false, Ordering::SeqCst);
        result
    })
    .await
}
