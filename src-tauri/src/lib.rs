//! PC Cleaner window: wiring of the core (`pccleaner-core`) to Tauri commands, events, the tray
//! and the window state. No cleaning logic lives here.

mod boot;
mod commands;
mod reporter;
mod state;
mod tray;
mod window;

use tauri::Manager;

use commands::{analysis, app, assistant, clean, drivers, scan, settings, startup};

pub fn run() {
    let Some(state) = boot::prepare() else { return };
    let auto_clean = state.cli.auto_clean;
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(state)
        .setup(move |a| {
            tray::create(a.handle())?;
            if let Some(w) = a.get_webview_window("main") {
                window::restore(&w, &a.state());
                // Automatic cleaning at sign-in runs in the tray without showing the window
                if auto_clean {
                    tray::hide_to_tray(a.handle(), true);
                } else {
                    w.show()?;
                }
            }
            Ok(())
        })
        .on_window_event(window::on_event)
        .invoke_handler(tauri::generate_handler![
            app::app_info, app::open_target, app::save_page, app::set_progress, app::exit_app, app::restart_app,
            app::hide_to_tray, app::show_window, app::window_visible, app::tray_texts, app::notify,
            scan::list_drives, scan::scan, scan::cleanup_history, clean::clean, analysis::analyse,
            startup::startup_list, startup::startup_set, startup::startup_remove,
            drivers::drivers_check, drivers::drivers_install,
            assistant::ai_status, assistant::ai_prepare, assistant::ai_chat, assistant::ai_stop, assistant::ai_install,
            settings::get_config, settings::save_config, settings::auto_clean_done, settings::clear_logs,
        ])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        eprintln!("PC Cleaner could not start: {e}");
    }
}
