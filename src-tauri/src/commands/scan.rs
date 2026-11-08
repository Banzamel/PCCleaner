use tauri::AppHandle;

use super::{blocking, lock, with_ctx};
use pccleaner_core::scan::session::ScanReport;
use pccleaner_core::scan::{full_scan, ScanScope};
use pccleaner_core::storage::history::{self, HistoryEntry};
use pccleaner_core::system::drives::{self, DriveInfo};
use pccleaner_core::AppResult;

#[tauri::command]
pub async fn list_drives(app: AppHandle) -> AppResult<Vec<DriveInfo>> {
    blocking(app, |_, st| Ok(drives::fixed_drives(&st.user.system_drive))).await
}

/// Full scan of the ticked drives. Deletes nothing; the result stays in memory for the cleanup.
#[tauri::command]
pub async fn scan(app: AppHandle, drives: Vec<String>) -> AppResult<ScanReport> {
    blocking(app, move |app, st| {
        let sys = st.user.system_drive.clone();
        let all: Vec<String> = drives::fixed_drives(&sys).into_iter().map(|d| d.letter).collect();
        let mut chosen = vec![sys.clone()];
        for d in drives.iter().map(|d| format!("{}:", d.trim_end_matches(['\\', ':']).to_uppercase())) {
            if all.contains(&d) && !chosen.contains(&d) {
                chosen.push(d);
            }
        }
        let scope = {
            let c = lock(&st.config)?;
            ScanScope { skip_categories: c.skip_categories.clone(), leftovers: c.scan_leftovers, large_files: c.scan_large_files }
        };
        let session = with_ctx(app, st, chosen, |ctx| full_scan(ctx, &scope));
        let report = session.report();
        st.log(&format!("Report: {}", serde_json::to_string(&report).unwrap_or_default()));
        *lock(&st.session)? = Some(session);
        Ok(report)
    })
    .await
}

#[tauri::command]
pub async fn cleanup_history(app: AppHandle) -> AppResult<Vec<HistoryEntry>> {
    blocking(app, |_, st| Ok(history::load(&st.data))).await
}
