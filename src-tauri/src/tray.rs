//! Icon in the notification area. It appears only while the window is hidden there (the X button
//! setting, automatic cleaning at sign-in).

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

const TRAY_ID: &str = "main";
static HINT_SHOWN: AtomicBool = AtomicBool::new(false);

/// Creates the hidden tray icon. Menu labels come from the window (translated) via [`set_labels`].
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open PC Cleaner", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &exit])?;
    let mut b = TrayIconBuilder::with_id(TRAY_ID).tooltip("PC Cleaner").menu(&menu).show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        b = b.icon(icon.clone());
    }
    let tray = b
        .on_menu_event(|app, e| match e.id().as_ref() {
            "open" => show(app),
            "exit" => crate::window::exit(app),
            _ => {}
        })
        .on_tray_icon_event(|t, e| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                show(t.app_handle());
            }
        })
        .build(app)?;
    tray.set_visible(false)?;
    Ok(())
}

pub fn set_labels(app: &AppHandle, open: &str, exit: &str) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let items = (MenuItem::with_id(app, "open", open, true, None::<&str>), MenuItem::with_id(app, "exit", exit, true, None::<&str>));
    if let (Ok(o), Ok(x)) = items {
        if let Ok(menu) = Menu::with_items(app, &[&o, &x]) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

pub fn set_tooltip(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(text));
    }
}

/// Hides the window; the program keeps running in the notification area. The first time it says so.
pub fn hide_to_tray(app: &AppHandle, quiet: bool) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_visible(true);
    }
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
    if !quiet && !HINT_SHOWN.swap(true, Ordering::SeqCst) {
        let _ = app.emit_to("main", "pcc://tray-hint", ());
    }
}

pub fn show(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_visible(false);
    }
}

pub fn notify(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

use tauri::Emitter;
