use tauri::{
    AppHandle, Manager,
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
                    let _ = app.emit("refresh-requested", ());
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

pub fn update_tray_tooltip(tray: &TrayIcon, tooltip: &str) -> Result<()> {
    tray.set_tooltip(Some(tooltip))?;
    Ok(())
}

// Helper function to determine tray icon color based on quota status
pub fn get_tray_status_color(usage_percentage: f64) -> &'static str {
    if usage_percentage >= 90.0 {
        "red"
    } else if usage_percentage >= 70.0 {
        "yellow"
    } else {
        "green"
    }
}
