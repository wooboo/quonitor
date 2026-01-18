use tauri::{
    AppHandle, Manager, Emitter,
    tray::{TrayIcon, TrayIconBuilder, MouseButton, MouseButtonState},
    menu::{Menu, MenuItem},
};
use crate::error::Result;

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "Refresh Now", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &refresh, &quit])?;

    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Quonitor - LLM Quota Monitor")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "refresh" => {
                    // Emit event to trigger refresh
                    if let Err(e) = app.emit("refresh-requested", ()) {
                        eprintln!("Failed to emit refresh event: {}", e);
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(tray)
}
