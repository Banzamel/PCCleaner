use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent};

use crate::state::AppState;
use crate::tray;

/// Creates the main window from tauri.conf.json with its WebView2 profile in `<program>\data\webview`
/// (by default WebView2 would keep it in %LOCALAPPDATA%\<identifier>).
pub fn create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let st = app.state::<AppState>();
    let config = app.config().app.windows.iter().find(|w| w.label == "main").cloned().unwrap_or_default();
    tauri::WebviewWindowBuilder::from_config(app, &config)?.data_directory(st.data.webview()).build()
}

/// Restores size and position from window.json when the window lands on a connected monitor;
/// the size follows a changed display scale. Otherwise the window stays centred.
pub fn restore(win: &WebviewWindow, st: &AppState) {
    let Ok(prefs) = st.window.lock().map(|p| p.clone()) else { return };
    if prefs.w <= 0 || prefs.h <= 0 {
        return;
    }
    let scale = win.scale_factor().unwrap_or(1.0);
    let (w, h) = prefs.scaled_size(scale);
    let visible = win.available_monitors().unwrap_or_default().iter().any(|m| {
        let (p, s) = (m.position(), m.size());
        prefs.x + 200 > p.x && prefs.x < p.x + s.width as i32 && prefs.y + 60 > p.y && prefs.y < p.y + s.height as i32
    });
    let min = (1080.0 * scale) as i32;
    if visible && w >= min {
        let _ = win.set_size(PhysicalSize::new(w as u32, h as u32));
        let _ = win.set_position(PhysicalPosition::new(prefs.x, prefs.y));
        if prefs.max {
            let _ = win.maximize();
        }
    }
}

/// Remembers the normal (not maximised) bounds while the window moves and resizes.
fn track(win: &tauri::Window, st: &AppState) {
    let maximized = win.is_maximized().unwrap_or(false);
    let minimized = win.is_minimized().unwrap_or(false);
    if let Ok(mut p) = st.window.lock() {
        p.max = maximized;
        p.scale = win.scale_factor().unwrap_or(1.0);
        if !maximized && !minimized {
            if let (Ok(pos), Ok(size)) = (win.outer_position(), win.outer_size()) {
                (p.x, p.y, p.w, p.h) = (pos.x, pos.y, size.width as i32, size.height as i32);
            }
        }
    }
}

pub fn save(st: &AppState) {
    if let Ok(p) = st.window.lock() {
        if let Err(e) = p.save(&st.data) {
            st.log(&format!("Saving window.json: {e}"));
        }
    }
}

/// The X button: hides to the tray when Settings say so; the program does not close while cleaning
/// (paused services, Explorer) or installing drivers.
pub fn on_event(win: &tauri::Window, event: &WindowEvent) {
    let app = win.app_handle();
    let st = app.state::<AppState>();
    match event {
        WindowEvent::Moved(_) | WindowEvent::Resized(_) => track(win, &st),
        WindowEvent::CloseRequested { api, .. } => {
            if st.exiting.load(Ordering::SeqCst) {
                save(&st);
                return;
            }
            let to_tray = st.config.lock().map(|c| c.close_to_tray).unwrap_or(false);
            if to_tray {
                api.prevent_close();
                tray::hide_to_tray(app, false);
                return;
            }
            for (busy, reason) in [(&st.cleaning, "cleaning"), (&st.installing_drivers, "drivers")] {
                if busy.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = app.emit("pcc://close-blocked", reason);
                    return;
                }
            }
            save(&st);
        }
        WindowEvent::Destroyed => {
            if let Ok(mut o) = st.ollama.lock() {
                o.stop();
            }
            st.log("End (window).");
        }
        _ => {}
    }
}

/// Really closes the program (tray Exit, update, restart).
pub fn exit(app: &AppHandle) {
    let st = app.state::<AppState>();
    st.exiting.store(true, Ordering::SeqCst);
    save(&st);
    if let Ok(mut o) = st.ollama.lock() {
        o.stop();
    }
    app.exit(0);
}
