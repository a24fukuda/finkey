// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

// キー設定（文字列またはプラットフォーム別オブジェクト）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShortcutKey {
    Simple(String),
    Platform {
        #[serde(default)]
        windows: Option<String>,
        #[serde(default, rename = "macos")]
        macos: Option<String>,
    },
}

impl ShortcutKey {
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

// ショートカットの構造体（アプリ名はJSONのキーから取得）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    /// アクション名（表示用）
    pub action: String,
    /// キー設定
    pub key: ShortcutKey,
    /// 説明
    #[serde(default)]
    pub description: String,
    /// 検索用タグ（ローマ字含む）
    #[serde(default)]
    pub tags: Vec<String>,
}

// フロントエンドに渡す正規化されたショートカット
#[derive(Debug, Clone, Serialize)]
pub struct NormalizedShortcut {
    pub app: String,
    pub action: String,
    pub key: String,
    pub description: String,
    pub tags: Vec<String>,
}

// プラットフォーム固有のアプリマッチング設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAppMatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
}

// プラットフォーム別設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<PlatformAppMatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos: Option<PlatformAppMatch>,
}

// アプリ設定（オブジェクト形式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRuleObject {
    pub display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<PlatformConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

// アプリ設定（文字列またはオブジェクト）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AppRule {
    Simple(String),
    Detailed(AppRuleObject),
}

// デフォルトアイコン
const DEFAULT_APP_ICON: &str = "📌";

// 正規化されたアプリルール（内部処理用）
#[derive(Debug, Clone, Serialize)]
pub struct NormalizedAppRule {
    pub display: String,
    pub process: Option<String>,
    pub window: Option<String>,
    pub icon: String,
}

impl AppRule {
    /// プラットフォームに応じて正規化されたルールを返す
    pub fn normalize(&self, is_macos: bool) -> NormalizedAppRule {
        match self {
            Self::Simple(name) => NormalizedAppRule {
                display: name.clone(),
                process: Some(name.clone()),
                window: Some(name.clone()),
                icon: DEFAULT_APP_ICON.to_string(),
            },
            Self::Detailed(obj) => {
                let icon = obj.icon.clone().unwrap_or_else(|| DEFAULT_APP_ICON.to_string());

                // プラットフォーム別設定があればそれを使用
                if let Some(ref platform) = obj.platform {
                    let platform_match = if is_macos {
                        platform.macos.as_ref()
                    } else {
                        platform.windows.as_ref()
                    };

                    if let Some(pm) = platform_match {
                        return NormalizedAppRule {
                            display: obj.display.clone(),
                            process: pm.process.clone(),
                            window: pm.window.clone(),
                            icon,
                        };
                    }
                }

                // プラットフォーム別設定がなければ共通設定を使用
                NormalizedAppRule {
                    display: obj.display.clone(),
                    process: obj.process.clone(),
                    window: obj.window.clone(),
                    icon,
                }
            }
        }
    }
}

// ショートカット設定ファイルの構造体（アプリ名 -> ショートカット配列）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    pub shortcuts: HashMap<String, Vec<Shortcut>>,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            shortcuts: get_default_shortcuts(),
        }
    }
}

// アプリ設定ファイルの構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsConfig {
    pub apps: Vec<AppRule>,
}

impl Default for AppsConfig {
    fn default() -> Self {
        Self {
            apps: get_default_apps(),
        }
    }
}

// デフォルトのアプリ設定（JSONファイルから読み込み）
const DEFAULT_APPS_JSON: &str = include_str!("../defaults/apps.json");

fn get_default_apps() -> Vec<AppRule> {
    serde_json::from_str::<AppsConfig>(DEFAULT_APPS_JSON)
        .map(|config| config.apps)
        .unwrap_or_default()
}

// 設定ディレクトリのパスを取得
fn get_config_dir() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    Some(config_dir.join("shortcut-finder"))
}

// ショートカット設定ファイルのパスを取得
fn get_shortcuts_config_path() -> Option<PathBuf> {
    Some(get_config_dir()?.join("shortcuts.json"))
}

// アプリ設定ファイルのパスを取得
fn get_apps_config_path() -> Option<PathBuf> {
    Some(get_config_dir()?.join("apps.json"))
}

// ショートカット設定を読み込む
fn load_shortcuts_config() -> ShortcutsConfig {
    if let Some(path) = get_shortcuts_config_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<ShortcutsConfig>(&content) {
                    return config;
                }
            }
        }
    }
    // ファイルがなければデフォルトを返し、設定ファイルを作成
    let config = ShortcutsConfig::default();
    let _ = save_shortcuts_config(&config);
    config
}

// ショートカット設定を保存
fn save_shortcuts_config(config: &ShortcutsConfig) -> Result<(), String> {
    let path = get_shortcuts_config_path().ok_or("設定ディレクトリが見つかりません")?;

    // ディレクトリを作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("ディレクトリ作成エラー: {e}"))?;
    }

    let json = serde_json::to_string_pretty(config).map_err(|e| format!("JSON変換エラー: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("ファイル書き込みエラー: {e}"))?;

    Ok(())
}

// アプリ設定を読み込む
fn load_apps_config() -> AppsConfig {
    if let Some(path) = get_apps_config_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppsConfig>(&content) {
                    return config;
                }
            }
        }
    }
    // ファイルがなければデフォルトを返し、設定ファイルを作成
    let config = AppsConfig::default();
    let _ = save_apps_config(&config);
    config
}

// アプリ設定を保存
fn save_apps_config(config: &AppsConfig) -> Result<(), String> {
    let path = get_apps_config_path().ok_or("設定ディレクトリが見つかりません")?;

    // ディレクトリを作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("ディレクトリ作成エラー: {e}"))?;
    }

    let json = serde_json::to_string_pretty(config).map_err(|e| format!("JSON変換エラー: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("ファイル書き込みエラー: {e}"))?;

    Ok(())
}

// デフォルトのショートカットデータ（JSONファイルから読み込み）
const DEFAULT_SHORTCUTS_JSON: &str = include_str!("../defaults/shortcuts.json");

fn get_default_shortcuts() -> HashMap<String, Vec<Shortcut>> {
    serde_json::from_str::<ShortcutsConfig>(DEFAULT_SHORTCUTS_JSON)
        .map(|config| config.shortcuts)
        .unwrap_or_default()
}

// アクティブウィンドウ情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveWindowInfo {
    pub process: Option<String>,
    pub window: Option<String>,
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
/// プロセス名またはウィンドウタイトルで部分一致（大文字小文字無視）
fn match_apps(info: &ActiveWindowInfo, apps: &[AppRule]) -> Vec<NormalizedAppRule> {
    let is_macos = cfg!(target_os = "macos");

    apps.iter()
        .filter_map(|rule| {
            let normalized = rule.normalize(is_macos);
            let mut matched = false;

            // プロセス名でマッチ
            if let (Some(ref rule_process), Some(ref info_process)) =
                (&normalized.process, &info.process)
            {
                if info_process
                    .to_lowercase()
                    .contains(&rule_process.to_lowercase())
                {
                    matched = true;
                }
            }

            // ウィンドウタイトルでマッチ
            if !matched {
                if let (Some(ref rule_window), Some(ref info_window)) =
                    (&normalized.window, &info.window)
                {
                    if info_window
                        .to_lowercase()
                        .contains(&rule_window.to_lowercase())
                    {
                        matched = true;
                    }
                }
            }

            if matched {
                Some(normalized)
            } else {
                None
            }
        })
        .collect()
}

// マッチしたアプリ情報を取得するコマンド
#[tauri::command]
fn get_matched_apps(info: Option<ActiveWindowInfo>) -> Vec<NormalizedAppRule> {
    let apps_config = load_apps_config();
    match info {
        Some(ref window_info) => match_apps(window_info, &apps_config.apps),
        None => vec![],
    }
}

// ショートカット一覧を取得するコマンド（プラットフォームに応じて正規化）
#[tauri::command]
fn get_shortcuts() -> Vec<NormalizedShortcut> {
    let is_macos = cfg!(target_os = "macos");
    let config = load_shortcuts_config();

    config
        .shortcuts
        .into_iter()
        .flat_map(|(app_name, shortcuts)| {
            shortcuts.into_iter().filter_map(move |shortcut| {
                // プラットフォームに応じたキーを取得
                let key = shortcut.key.get_key(is_macos)?;
                // キーが"-"の場合は対象外
                if key == "-" {
                    return None;
                }
                Some(NormalizedShortcut {
                    app: app_name.clone(),
                    action: shortcut.action,
                    key,
                    description: shortcut.description,
                    tags: shortcut.tags,
                })
            })
        })
        .collect()
}

// 設定ファイルのパスを取得するコマンド
#[tauri::command]
fn get_config_file_path() -> Option<String> {
    get_shortcuts_config_path().map(|p| p.to_string_lossy().to_string())
}

// ショートカットを保存するコマンド
#[tauri::command]
fn save_shortcuts(shortcuts: HashMap<String, Vec<Shortcut>>) -> Result<(), String> {
    let config = ShortcutsConfig { shortcuts };
    save_shortcuts_config(&config)
}

// ショートカット設定ファイルを開くコマンド
#[tauri::command]
fn open_config_file() -> Result<(), String> {
    let path = get_shortcuts_config_path().ok_or("設定ファイルのパスが見つかりません")?;

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

// アプリ設定ファイルを開くコマンド
#[tauri::command]
fn open_apps_config_file() -> Result<(), String> {
    // ファイルが存在しない場合は作成する
    let _ = load_apps_config();

    let path = get_apps_config_path().ok_or("アプリ設定ファイルのパスが見つかりません")?;

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

fn create_system_tray() -> SystemTray {
    let show = CustomMenuItem::new("show".to_string(), "ウィンドウを表示");
    let shortcuts_config =
        CustomMenuItem::new("shortcuts_config".to_string(), "ショートカット設定を開く");
    let apps_config = CustomMenuItem::new("apps_config".to_string(), "アプリ設定を開く");
    let quit = CustomMenuItem::new("quit".to_string(), "終了");

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(shortcuts_config)
        .add_item(apps_config)
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
                "shortcuts_config" => {
                    let _ = open_config_file();
                }
                "apps_config" => {
                    let _ = open_apps_config_file();
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
            save_shortcuts,
            open_config_file,
            open_apps_config_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
