// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{
    AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};

// デフォルトアイコン
const DEFAULT_APP_ICON: &str = "📌";

// キー設定（文字列またはプラットフォーム別オブジェクト）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyBinding {
    Simple(String),
    Platform {
        #[serde(default)]
        windows: Option<String>,
        #[serde(default, rename = "macos")]
        macos: Option<String>,
    },
}

impl KeyBinding {
    /// プラットフォームに応じたキーを取得
    pub fn get_key(&self, is_macos: bool) -> Option<String> {
        match self {
            Self::Simple(key) => Some(key.clone()),
            Self::Platform { windows, macos } => {
                if is_macos {
                    macos.clone()
                } else {
                    windows.clone()
                }
            }
        }
    }
}

// バインド設定（文字列または配列）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AppBind {
    Single(String),
    Multiple(Vec<String>),
}

impl AppBind {
    /// バインド値のリストを取得
    pub fn get_binds(&self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s.clone()],
            Self::Multiple(v) => v.clone(),
        }
    }
}

// キーバインド設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub action: String,
    pub key: KeyBinding,
    #[serde(default)]
    pub tags: Vec<String>,
}

// OS種別（windows または macos のみ）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OsType {
    Windows,
    #[serde(rename = "macos")]
    MacOS,
}

impl OsType {
    /// OS種別から表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOS => "macOS",
        }
    }

    /// 現在のプラットフォームと一致するか
    pub fn is_current_platform(&self) -> bool {
        match self {
            Self::Windows => cfg!(target_os = "windows"),
            Self::MacOS => cfg!(target_os = "macos"),
        }
    }
}

// アプリ設定（統合形式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub icon: Option<String>,
    /// アプリ名（osが指定されている場合は不要）
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub bind: Option<AppBind>,
    /// OS種別（windows または macos）。指定時はnameとbindは不要
    #[serde(default)]
    pub os: Option<OsType>,
    #[serde(default)]
    pub keybindings: Vec<Keybinding>,
}

impl AppConfig {
    /// アイコンを取得（未設定の場合はデフォルト）
    pub fn get_icon(&self) -> String {
        self.icon
            .clone()
            .unwrap_or_else(|| DEFAULT_APP_ICON.to_string())
    }

    /// 表示名を取得（osがあればOS名、なければname）
    pub fn get_name(&self) -> String {
        if let Some(ref os) = self.os {
            os.display_name().to_string()
        } else {
            self.name.clone().unwrap_or_default()
        }
    }

    /// バインド値のリストを取得（未設定の場合はnameを使用）
    pub fn get_binds(&self) -> Vec<String> {
        match &self.bind {
            Some(bind) => bind.get_binds(),
            None => vec![self.get_name()],
        }
    }

    /// 現在のプラットフォームで有効かどうか
    /// osが指定されていない場合は常に有効、指定されている場合は一致時のみ有効
    pub fn is_available(&self) -> bool {
        match &self.os {
            Some(os) => os.is_current_platform(),
            None => true,
        }
    }
}

// フロントエンドに渡す正規化されたショートカット
#[derive(Debug, Clone, Serialize)]
pub struct NormalizedShortcut {
    pub app: String,
    pub icon: String,
    pub action: String,
    pub key: String,
    pub tags: Vec<String>,
}

// 正規化されたアプリ情報（フロントエンドに渡す用）
#[derive(Debug, Clone, Serialize)]
pub struct NormalizedApp {
    pub name: String,
    pub icon: String,
}

// アクティブウィンドウ情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveWindowInfo {
    pub process: Option<String>,
    pub window: Option<String>,
}

// テーマ設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSetting {
    System,
    Light,
    Dark,
}

impl Default for ThemeSetting {
    fn default() -> Self {
        Self::System
    }
}

// アプリ設定（settings.json）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub theme: ThemeSetting,
}

// デフォルトのキーバインド設定（JSONファイルから読み込み）
const DEFAULT_KEYBINDINGS_JSON: &str = include_str!("../defaults/keybindings.json");

fn get_default_keybindings() -> Vec<AppConfig> {
    serde_json::from_str::<Vec<AppConfig>>(DEFAULT_KEYBINDINGS_JSON).unwrap_or_default()
}

// 設定ディレクトリのパスを取得
fn get_config_dir() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    Some(config_dir.join("shortcut-finder"))
}

// キーバインド設定ファイルのパスを取得
fn get_keybindings_config_path() -> Option<PathBuf> {
    Some(get_config_dir()?.join("keybindings.json"))
}

// アプリ設定ファイルのパスを取得
fn get_settings_path() -> Option<PathBuf> {
    Some(get_config_dir()?.join("settings.json"))
}

// アプリ設定を読み込む（ファイルがなければ作成）
fn load_settings() -> AppSettings {
    if let Some(path) = get_settings_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    return settings;
                }
            }
        }
    }
    // ファイルがなければデフォルト設定を作成して保存
    let settings = AppSettings::default();
    let _ = save_settings(&settings);
    settings
}

// アプリ設定を保存
fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path().ok_or("設定ディレクトリが見つかりません")?;

    // ディレクトリを作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("ディレクトリ作成エラー: {e}"))?;
    }

    let json = serde_json::to_string_pretty(settings).map_err(|e| format!("JSON変換エラー: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("ファイル書き込みエラー: {e}"))?;

    Ok(())
}

// キーバインド設定を読み込む
fn load_keybindings_config() -> Vec<AppConfig> {
    if let Some(path) = get_keybindings_config_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<Vec<AppConfig>>(&content) {
                    return config;
                }
            }
        }
    }
    // ファイルがなければデフォルトを返し、設定ファイルを作成
    let config = get_default_keybindings();
    let _ = save_keybindings_config(&config);
    config
}

// キーバインド設定を保存
fn save_keybindings_config(config: &Vec<AppConfig>) -> Result<(), String> {
    let path = get_keybindings_config_path().ok_or("設定ディレクトリが見つかりません")?;

    // ディレクトリを作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("ディレクトリ作成エラー: {e}"))?;
    }

    let json = serde_json::to_string_pretty(config).map_err(|e| format!("JSON変換エラー: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("ファイル書き込みエラー: {e}"))?;

    Ok(())
}

// 前回アクティブだったアプリ情報を保持
static LAST_ACTIVE_APP: Mutex<Option<ActiveWindowInfo>> = Mutex::new(None);
// ウィンドウが表示中かどうか
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
mod active_window {
    use super::ActiveWindowInfo;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{
        GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    };

    /// アクティブなウィンドウの情報を取得（自分自身を除外）
    #[allow(unsafe_code)]
    pub fn get_active_window_info() -> Option<ActiveWindowInfo> {
        // SAFETY: Windows APIの呼び出しに必要
        unsafe {
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&raw mut process_id));

            if process_id == 0 {
                return None;
            }

            // 自分自身のプロセスIDと比較して除外
            let current_pid = GetCurrentProcessId();
            if process_id == current_pid {
                return None;
            }

            // プロセス名を取得
            let process_name = {
                let process_handle = OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                    false,
                    process_id,
                )
                .ok();

                process_handle.and_then(|handle| {
                    let mut buffer = [0u16; 260];
                    let len = GetModuleBaseNameW(handle, None, &mut buffer);
                    if len == 0 {
                        None
                    } else {
                        let name = String::from_utf16_lossy(&buffer[..len as usize]);
                        Some(
                            name.trim_end_matches(".exe")
                                .trim_end_matches(".EXE")
                                .to_string(),
                        )
                    }
                })
            };

            // ウィンドウタイトルを取得
            let window_title = {
                let len = GetWindowTextLengthW(hwnd);
                if len > 0 {
                    let mut buffer = vec![0u16; (len + 1) as usize];
                    let actual_len = GetWindowTextW(hwnd, &mut buffer);
                    if actual_len > 0 {
                        Some(String::from_utf16_lossy(&buffer[..actual_len as usize]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            Some(ActiveWindowInfo {
                process: process_name,
                window: window_title,
            })
        }
    }
}

#[cfg(target_os = "macos")]
mod active_window {
    use super::ActiveWindowInfo;
    /// macOS: ダミー実装
    pub fn get_active_window_info() -> Option<ActiveWindowInfo> {
        None
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod active_window {
    use super::ActiveWindowInfo;
    /// その他のOS: ダミー実装
    pub fn get_active_window_info() -> Option<ActiveWindowInfo> {
        None
    }
}

// 前回のアクティブアプリを更新する
fn update_last_active_app() {
    if let Some(info) = active_window::get_active_window_info() {
        if let Ok(mut last_app) = LAST_ACTIVE_APP.lock() {
            *last_app = Some(info);
        }
    }
}

// 前回のアクティブアプリ情報を取得する
fn get_last_active_app() -> Option<ActiveWindowInfo> {
    LAST_ACTIVE_APP.lock().ok()?.clone()
}

// ウィンドウの表示/非表示を切り替え
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        if window.is_visible().unwrap_or(false) {
            WINDOW_VISIBLE.store(false, Ordering::SeqCst);
            let _ = window.hide();
        } else {
            // 保存しておいた前回のアクティブアプリを使用
            let active_app = get_last_active_app();

            WINDOW_VISIBLE.store(true, Ordering::SeqCst);
            let _ = window.center();
            let _ = window.show();
            let _ = window.set_focus();
            // フロントエンドに通知（アクティブアプリ名を含む）
            let _ = window.emit("window-shown", active_app);
        }
    }
}

// ウィンドウを非表示
fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        WINDOW_VISIBLE.store(false, Ordering::SeqCst);
        let _ = window.hide();
        let _ = window.emit("window-hidden", ());
    }
}

// バックグラウンドでアクティブウィンドウを監視するスレッドを開始
fn start_active_window_monitor() {
    thread::spawn(|| {
        loop {
            let visible = WINDOW_VISIBLE.load(Ordering::SeqCst);
            // ウィンドウが非表示の時だけアクティブアプリを更新
            if !visible {
                update_last_active_app();
            }
            // 200msごとに監視
            thread::sleep(Duration::from_millis(200));
        }
    });
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

// アクティブなウィンドウ情報を取得するコマンド
#[tauri::command]
fn get_active_app() -> Option<ActiveWindowInfo> {
    active_window::get_active_window_info()
}

/// アクティブウィンドウにマッチするアプリを検索
/// プロセス名またはウィンドウタイトルで完全一致（大文字小文字無視）
fn match_apps(info: &ActiveWindowInfo, apps: &[AppConfig]) -> Vec<NormalizedApp> {
    apps.iter()
        .filter_map(|app| {
            let binds = app.get_binds();
            let mut matched = false;

            for bind in &binds {
                // プロセス名で完全一致
                if let Some(ref info_process) = info.process {
                    if info_process.to_lowercase() == bind.to_lowercase() {
                        matched = true;
                        break;
                    }
                }

                // ウィンドウタイトルで完全一致
                if let Some(ref info_window) = info.window {
                    if info_window.to_lowercase() == bind.to_lowercase() {
                        matched = true;
                        break;
                    }
                }
            }

            if matched {
                Some(NormalizedApp {
                    name: app.get_name(),
                    icon: app.get_icon(),
                })
            } else {
                None
            }
        })
        .collect()
}

// マッチしたアプリ情報を取得するコマンド
#[tauri::command]
fn get_matched_apps(info: Option<ActiveWindowInfo>) -> Vec<NormalizedApp> {
    let config = load_keybindings_config();
    match info {
        Some(ref window_info) => match_apps(window_info, &config),
        None => vec![],
    }
}

// ショートカット一覧を取得するコマンド（プラットフォームに応じて正規化）
#[tauri::command]
fn get_shortcuts() -> Vec<NormalizedShortcut> {
    let is_macos = cfg!(target_os = "macos");
    let config = load_keybindings_config();

    config
        .into_iter()
        // 現在のプラットフォームで有効なアプリのみ
        .filter(|app| app.is_available())
        .flat_map(|app| {
            let app_name = app.get_name();
            let app_icon = app.get_icon();
            app.keybindings.into_iter().filter_map(move |kb| {
                // プラットフォームに応じたキーを取得
                let key = kb.key.get_key(is_macos)?;
                // キーが"-"の場合は対象外
                if key == "-" {
                    return None;
                }
                Some(NormalizedShortcut {
                    app: app_name.clone(),
                    icon: app_icon.clone(),
                    action: kb.action,
                    key,
                    tags: kb.tags,
                })
            })
        })
        .collect()
}

// 設定ファイルのパスを取得するコマンド
#[tauri::command]
fn get_config_file_path() -> Option<String> {
    get_keybindings_config_path().map(|p| p.to_string_lossy().to_string())
}

// キーバインド設定ファイルを開くコマンド
#[tauri::command]
fn open_config_file() -> Result<(), String> {
    let path = get_keybindings_config_path().ok_or("設定ファイルのパスが見つかりません")?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", path.to_string_lossy().as_ref()])
            .spawn()
            .map_err(|e| format!("ファイルを開けませんでした: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("ファイルを開けませんでした: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("ファイルを開けませんでした: {e}"))?;
    }

    Ok(())
}

// テーマ設定を取得
#[tauri::command]
fn get_theme_setting() -> String {
    let settings = load_settings();
    match settings.theme {
        ThemeSetting::System => "system".to_string(),
        ThemeSetting::Light => "light".to_string(),
        ThemeSetting::Dark => "dark".to_string(),
    }
}

// テーマ設定を保存
#[tauri::command]
fn set_theme_setting(theme: String) -> Result<(), String> {
    let mut settings = load_settings();
    settings.theme = match theme.as_str() {
        "light" => ThemeSetting::Light,
        "dark" => ThemeSetting::Dark,
        _ => ThemeSetting::System,
    };
    save_settings(&settings)
}

// システムテーマを取得（ウィンドウから）
#[tauri::command]
fn get_system_theme(window: tauri::Window) -> String {
    match window.theme() {
        Ok(tauri::Theme::Dark) => "dark".to_string(),
        Ok(tauri::Theme::Light) => "light".to_string(),
        _ => "light".to_string(),
    }
}

fn create_system_tray() -> SystemTray {
    let show = CustomMenuItem::new("show".to_string(), "ウィンドウを表示");
    let config = CustomMenuItem::new("config".to_string(), "設定を開く");
    let quit = CustomMenuItem::new("quit".to_string(), "終了");

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(config)
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
                "config" => {
                    let _ = open_config_file();
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

            // バックグラウンドでアクティブウィンドウを監視開始
            start_active_window_monitor();

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
                WINDOW_VISIBLE.store(true, Ordering::SeqCst);
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
                        WINDOW_VISIBLE.store(false, Ordering::SeqCst);
                        let _ = event.window().hide();
                    }
                }
                // 閉じるボタンでアプリを終了せず、ウィンドウを非表示にする
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    WINDOW_VISIBLE.store(false, Ordering::SeqCst);
                    let _ = event.window().hide();
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            hide_main_window,
            get_platform,
            get_active_app,
            get_matched_apps,
            get_shortcuts,
            get_config_file_path,
            open_config_file,
            get_theme_setting,
            set_theme_setting,
            get_system_theme
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
