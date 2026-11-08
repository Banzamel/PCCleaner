use std::path::Path;

use serde::Serialize;
use tauri::window::{ProgressBarState, ProgressBarStatus};
use tauri::{AppHandle, Manager, State};

use super::{exe, lock, version};
use crate::state::AppState;
use crate::{tray, window};
use pccleaner_core::ai::models;
use pccleaner_core::system::{command, explorer, locale};
use pccleaner_core::AppResult;

/// How this run was started, with command-line parameters already merged over the settings.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub language: String,
    pub model: String,
    pub preview: bool,
    pub no_ai: bool,
    pub no_update_check: bool,
    pub auto_clean: bool,
    pub other_user: bool,
    /// No uninstaller next to the program: updates are downloaded by hand.
    pub portable: bool,
    pub log_file: String,
    pub log_dir: String,
    pub system_drive: String,
    /// auto, light or dark
    pub theme: String,
    pub page: String,
    pub drives: Vec<String>,
}

#[tauri::command]
pub fn app_info(app: AppHandle, st: State<'_, AppState>) -> AppResult<AppInfo> {
    let c = lock(&st.config)?.clone();
    let w = lock(&st.window)?.clone();
    let language = st.cli.language.clone().or(Some(c.language.clone()).filter(|l| l != "auto")).unwrap_or_else(|| locale::ui_language().to_string());
    let model = st.cli.model.clone().or(Some(c.model.clone()).filter(|m| !m.is_empty())).unwrap_or_else(|| models::installer_model(&st.app_dir));
    let theme = if !c.theme.is_empty() { c.theme.clone() } else if !w.theme.is_empty() { w.theme.clone() } else { "auto".into() };
    st.log(&format!("Start (window). Model: {model}, Preview: {}, NoAI: {}, Language: {language}", st.cli.preview || c.preview, st.cli.no_ai || !c.assistant));
    Ok(AppInfo {
        version: version(&app),
        language,
        model,
        preview: st.cli.preview || c.preview,
        no_ai: st.cli.no_ai || !c.assistant,
        no_update_check: st.cli.no_update_check || !c.check_updates || st.cli.auto_clean,
        auto_clean: st.cli.auto_clean,
        other_user: st.user.other_user,
        portable: !st.app_dir.join("uninstall.exe").is_file(),
        log_file: st.logger.path.to_string_lossy().to_string(),
        log_dir: st.data.logs().to_string_lossy().to_string(),
        system_drive: st.user.system_drive.clone(),
        theme,
        page: w.page,
        drives: st.cli.drives.clone(),
    })
}

/// Opens things for the user: a file in Explorer, the logs folder, the log, Device Manager, a settings page.
#[tauri::command]
pub fn open_target(st: State<'_, AppState>, kind: String, path: Option<String>) {
    match (kind.as_str(), path) {
        ("reveal", Some(p)) if Path::new(&p).exists() => explorer::reveal(Path::new(&p)),
        ("logs", _) => {
            explorer::shell_open(&st.data.logs().to_string_lossy());
        }
        ("log", _) => explorer::notepad(&st.logger.path),
        ("devices", _) => {
            explorer::shell_open("devmgmt.msc");
        }
        ("windows-update", _) => {
            explorer::shell_open("ms-settings:windowsupdate-optionalupdates");
        }
        ("url", Some(u)) if u.starts_with("https://") => {
            explorer::shell_open(&u);
        }
        _ => {}
    }
}

/// The last page, kept in window.json (the plan and cleanup pages fall back to the overview).
#[tauri::command]
pub fn save_page(st: State<'_, AppState>, page: String) {
    if let Ok(mut w) = st.window.lock() {
        w.page = if matches!(page.as_str(), "plan" | "cleanup") { "overview".into() } else { page };
    }
}

/// Taskbar progress: state none, indeterminate, normal or error; value 0-100.
#[tauri::command]
pub fn set_progress(app: AppHandle, state: String, value: u64) {
    let status = match state.as_str() {
        "indeterminate" => ProgressBarStatus::Indeterminate,
        "normal" => ProgressBarStatus::Normal,
        "error" => ProgressBarStatus::Error,
        _ => ProgressBarStatus::None,
    };
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_progress_bar(ProgressBarState { status: Some(status), progress: Some(value.min(100)) });
    }
}

#[tauri::command]
pub fn exit_app(app: AppHandle) {
    window::exit(&app);
}

/// "Save and restart": the new process inherits the administrator rights, so UAC does not ask again.
#[tauri::command]
pub fn restart_app(app: AppHandle, st: State<'_, AppState>) {
    let mut args = vec!["--sid".to_string(), st.user.sid.clone()];
    if st.cli.preview {
        args.push("--preview".into());
    }
    match command::hidden(&exe().to_string_lossy()).args(&args).spawn() {
        Ok(_) => st.log("Restarting after settings change."),
        Err(e) => st.log(&format!("Restart: {e}")),
    }
    window::exit(&app);
}

#[tauri::command]
pub fn hide_to_tray(app: AppHandle, quiet: bool) {
    tray::hide_to_tray(&app, quiet);
}

#[tauri::command]
pub fn window_visible(app: AppHandle) -> bool {
    app.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false)
}

#[tauri::command]
pub fn show_window(app: AppHandle) {
    tray::show(&app);
}

/// Translated tray texts and a notification (automatic cleaning, the tray hint).
#[tauri::command]
pub fn tray_texts(app: AppHandle, open: String, exit: String, tooltip: String) {
    tray::set_labels(&app, &open, &exit);
    tray::set_tooltip(&app, &tooltip);
}

#[tauri::command]
pub fn notify(app: AppHandle, title: String, body: String) {
    tray::notify(&app, &title, &body);
}
