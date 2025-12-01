// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};

// ウィンドウの表示/非表示を切り替え
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.center();
            let _ = window.show();
            let _ = window.set_focus();
            // フロントエンドに通知
            let _ = window.emit("window-shown", ());
        }
    }
}

// ウィンドウを非表示
fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        let _ = window.hide();
        let _ = window.emit("window-hidden", ());
    }
}

// ウィンドウを非表示にするコマンド
#[tauri::command]
fn hide_main_window(app: AppHandle) {
    hide_window(&app);
}

// プラットフォームを取得するコマンド
#[tauri::command]
fn get_platform() -> String {
    if cfg!(target_os = "macos") {
        "darwin".to_string()
    } else if cfg!(target_os = "windows") {
        "win32".to_string()
    } else {
        "linux".to_string()
    }
}

fn create_system_tray() -> SystemTray {
    let show = CustomMenuItem::new("show".to_string(), "ウィンドウを表示");
    let quit = CustomMenuItem::new("quit".to_string(), "終了");

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

fn main() {
    tauri::Builder::default()
        .system_tray(create_system_tray())
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                toggle_window(app);
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    toggle_window(app);
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .setup(|app| {
            let app_handle = app.handle();

            // グローバルショートカットを登録
            // Mac: Cmd+Shift+K, Windows/Linux: Ctrl+Shift+K
            let shortcut = if cfg!(target_os = "macos") {
                "Command+Shift+K"
            } else {
                "Ctrl+Shift+K"
            };

            let app_handle_clone = app_handle.clone();
            if let Err(e) = app.global_shortcut_manager().register(shortcut, move || {
                toggle_window(&app_handle_clone);
            }) {
                eprintln!("Warning: Failed to register global shortcut ({shortcut}): {e:?}");
            }

            // Escキーでウィンドウを閉じる
            if let Err(e) = app.global_shortcut_manager().register("Escape", move || {
                if let Some(window) = app_handle.get_window("main") {
                    if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false)
                    {
                        hide_window(&app_handle);
                    }
                }
            }) {
                eprintln!("Warning: Failed to register Escape shortcut: {e:?}");
            }

            // 初期表示
            if let Some(window) = app.get_window("main") {
                let _ = window.center();
                let _ = window.show();
                let _ = window.set_focus();
                // devtoolsを閉じる
                #[cfg(debug_assertions)]
                window.close_devtools();
            }

            Ok(())
        })
        .on_window_event(|event| {
            match event.event() {
                // フォーカスを失ったらウィンドウを非表示
                WindowEvent::Focused(focused) => {
                    if !focused {
                        let _ = event.window().hide();
                    }
                }
                // 閉じるボタンでアプリを終了せず、ウィンドウを非表示にする
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = event.window().hide();
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![hide_main_window, get_platform])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
