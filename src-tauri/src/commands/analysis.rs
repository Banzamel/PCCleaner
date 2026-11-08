use tauri::AppHandle;

use super::{blocking, reporter};
use pccleaner_core::analysis::{self, Analysis};
use pccleaner_core::AppResult;

/// "What takes up space" for one drive letter (`C`). Read-only.
#[tauri::command]
pub async fn analyse(app: AppHandle, letter: String) -> AppResult<Analysis> {
    blocking(app, move |app, st| {
        let letter: String = letter.chars().take(1).collect::<String>().to_uppercase();
        let out = reporter(app, st);
        Ok(analysis::analyse(&letter, &st.user.system_drive, &out))
    })
    .await
}
