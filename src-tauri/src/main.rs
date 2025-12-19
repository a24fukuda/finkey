// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime};
use tauri::{
    AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};

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
    pub key: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

// OS種別（windows または macos のみ）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    /// アイコンを取得（未設定の場合は空文字、フロントエンドでデフォルト適用）
    pub fn get_icon(&self) -> String {
        self.icon.clone().unwrap_or_default()
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSetting {
    #[default]
    System,
    Light,
    Dark,
}

// オーバーレイの位置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OverlayPosition {
    pub x: Option<i32>,
    pub y: Option<i32>,
}

// デフォルト設定の構造体（defaults/settings.json用、すべてのフィールドが必須）
#[derive(Debug, Clone, Deserialize)]
struct DefaultSettings {
    theme: ThemeSetting,
    hotkey: String,
    overlay_duration: u32,
}

// デフォルト設定のキャッシュ
static DEFAULT_SETTINGS_CACHE: OnceLock<DefaultSettings> = OnceLock::new();

// アプリ設定（settings.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: ThemeSetting,
    /// アプリ起動のホットキー（例: "Ctrl+Shift+K"）
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    /// オーバーレイ表示時間（秒）
    #[serde(default = "default_overlay_duration")]
    pub overlay_duration: u32,
    /// オーバーレイの位置（ドラッグで移動した場合に保存）
    #[serde(default)]
    pub overlay_position: OverlayPosition,
}

// デフォルト設定のJSONを読み込み（プラットフォーム別）
#[cfg(target_os = "windows")]
const DEFAULT_SETTINGS_JSON: &str = include_str!("../defaults/windows/settings.json");
#[cfg(target_os = "macos")]
const DEFAULT_SETTINGS_JSON: &str = include_str!("../defaults/macos/settings.json");

/// defaults/settings.json から設定を取得（初回のみパースしてキャッシュ）
fn get_defaults() -> &'static DefaultSettings {
    DEFAULT_SETTINGS_CACHE.get_or_init(|| {
        serde_json::from_str::<DefaultSettings>(DEFAULT_SETTINGS_JSON)
            .expect("defaults/settings.json のパースに失敗しました。ファイルが正しいJSON形式か確認してください。")
    })
}

fn default_theme() -> ThemeSetting {
    get_defaults().theme.clone()
}

fn default_hotkey() -> String {
    get_defaults().hotkey.clone()
}

fn default_overlay_duration() -> u32 {
    get_defaults().overlay_duration
}

/// ショートカットキー文字列を正規化（Tauri API用）
/// スペースあり/なし両方の入力形式を受け付け、スペースなし形式に変換
fn normalize_hotkey_for_tauri(key: &str) -> String {
    // " + " → "+" に変換（スペースあり→なし）
    key.replace(" + ", "+")
}

/// ショートカットキー文字列を正規化（表示用）
/// スペースあり/なし両方の入力形式を受け付け、スペースあり形式に変換
fn normalize_key_for_display(key: &str) -> String {
    // 既にスペースありの場合はそのまま
    if key.contains(" + ") {
        return key.to_string();
    }
    // "+" → " + " に変換（スペースなし→あり）
    // ただし "++" のような連続は考慮（Ctrl+Shift++ など）
    let mut result = String::new();
    let chars: Vec<char> = key.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '+' {
            // 次の文字も + の場合はキー自体が + なのでスキップ
            if i + 1 < chars.len() && chars[i + 1] == '+' {
                result.push_str(" + +");
                i += 2;
                continue;
            }
            result.push_str(" + ");
        } else {
            result.push(chars[i]);
        }
        i += 1;
    }
    result
}

impl Default for AppSettings {
    fn default() -> Self {
        let defaults = get_defaults();
        Self {
            theme: defaults.theme.clone(),
            hotkey: defaults.hotkey.clone(),
            overlay_duration: defaults.overlay_duration,
            overlay_position: OverlayPosition::default(),
        }
    }
}

// デフォルトのキーバインド設定（JSONファイルから読み込み、プラットフォーム別）
#[cfg(target_os = "windows")]
const DEFAULT_KEYBINDINGS_JSON: &str = include_str!("../defaults/windows/keybindings.json");
#[cfg(target_os = "macos")]
const DEFAULT_KEYBINDINGS_JSON: &str = include_str!("../defaults/macos/keybindings.json");

fn get_default_keybindings() -> Vec<AppConfig> {
    serde_json::from_str::<Vec<AppConfig>>(DEFAULT_KEYBINDINGS_JSON)
        .expect("defaults/keybindings.json のパースに失敗しました。ファイルが正しいJSON形式か確認してください。")
}

// 設定ディレクトリのパスを取得
fn get_config_dir() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    Some(config_dir.join("finkey"))
}

// キーバインド設定ファイルのパスを取得
fn get_keybindings_config_path() -> Option<PathBuf> {
    Some(get_config_dir()?.join("keybindings.json"))
}

// アプリ設定ファイルのパスを取得
fn get_settings_path() -> Option<PathBuf> {
    Some(get_config_dir()?.join("settings.json"))
}

// アプリ設定を読み込む（キャッシュ付き）
fn load_settings() -> AppSettings {
    let path = match get_settings_path() {
        Some(p) => p,
        None => return AppSettings::default(),
    };

    let current_modified = get_file_modified_time(&path);

    // キャッシュをチェック
    if let Ok(cache_guard) = SETTINGS_CACHE.lock() {
        if let Some(ref cache) = *cache_guard {
            // タイムスタンプが同じならキャッシュを返す
            if cache.last_modified == current_modified && current_modified.is_some() {
                return cache.data.clone();
            }
        }
    }

    // ファイルを読み込む
    let settings = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<AppSettings>(&content).ok())
            .unwrap_or_else(|| {
                let default = AppSettings::default();
                let _ = save_settings(&default);
                default
            })
    } else {
        let default = AppSettings::default();
        let _ = save_settings(&default);
        default
    };

    // キャッシュを更新
    if let Ok(mut cache_guard) = SETTINGS_CACHE.lock() {
        *cache_guard = Some(SettingsCache {
            data: settings.clone(),
            last_modified: get_file_modified_time(&path),
        });
    }

    settings
}

// アプリ設定を保存
fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path().ok_or("設定ディレクトリが見つかりません")?;

    // ディレクトリを作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("ディレクトリ作成エラー: {e}"))?;
    }

    let json =
        serde_json::to_string_pretty(settings).map_err(|e| format!("JSON変換エラー: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("ファイル書き込みエラー: {e}"))?;

    // キャッシュを更新
    if let Ok(mut cache_guard) = SETTINGS_CACHE.lock() {
        *cache_guard = Some(SettingsCache {
            data: settings.clone(),
            last_modified: get_file_modified_time(&path),
        });
    }

    Ok(())
}

/// ファイルの最終更新時刻を取得
fn get_file_modified_time(path: &PathBuf) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

// キーバインド設定を読み込む（キャッシュ付き）
fn load_keybindings_config() -> Vec<AppConfig> {
    let path = match get_keybindings_config_path() {
        Some(p) => p,
        None => {
            let config = get_default_keybindings();
            return config;
        }
    };

    let current_modified = get_file_modified_time(&path);

    // キャッシュをチェック
    if let Ok(cache_guard) = KEYBINDINGS_CACHE.lock() {
        if let Some(ref cache) = *cache_guard {
            // タイムスタンプが同じならキャッシュを返す
            if cache.last_modified == current_modified && current_modified.is_some() {
                return cache.data.clone();
            }
        }
    }

    // ファイルを読み込む
    let config = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<Vec<AppConfig>>(&content).ok())
            .unwrap_or_else(|| {
                let default = get_default_keybindings();
                let _ = save_keybindings_config(&default);
                default
            })
    } else {
        let default = get_default_keybindings();
        let _ = save_keybindings_config(&default);
        default
    };

    // キャッシュを更新
    if let Ok(mut cache_guard) = KEYBINDINGS_CACHE.lock() {
        *cache_guard = Some(KeybindingsCache {
            data: config.clone(),
            last_modified: get_file_modified_time(&path),
        });
    }

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

    // キャッシュをクリア（次回読み込み時に再取得）
    if let Ok(mut cache_guard) = KEYBINDINGS_CACHE.lock() {
        *cache_guard = None;
    }

    Ok(())
}

// キャッシュ用の構造体
struct KeybindingsCache {
    data: Vec<AppConfig>,
    last_modified: Option<SystemTime>,
}

struct SettingsCache {
    data: AppSettings,
    last_modified: Option<SystemTime>,
}

// キャッシュ
static KEYBINDINGS_CACHE: Mutex<Option<KeybindingsCache>> = Mutex::new(None);
static SETTINGS_CACHE: Mutex<Option<SettingsCache>> = Mutex::new(None);

// 前回アクティブだったアプリ情報を保持
static LAST_ACTIVE_APP: Mutex<Option<ActiveWindowInfo>> = Mutex::new(None);
// 前回アクティブだったウィンドウのHWND（Windows用）
#[cfg(target_os = "windows")]
static LAST_ACTIVE_HWND: Mutex<Option<isize>> = Mutex::new(None);
// ウィンドウが表示中かどうか
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
mod active_window {
    use super::{ActiveWindowInfo, LAST_ACTIVE_HWND};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{
        GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        SetForegroundWindow,
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

            // HWNDを保存
            if let Ok(mut last_hwnd) = LAST_ACTIVE_HWND.lock() {
                *last_hwnd = Some(hwnd.0 as isize);
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
                    let result = if len == 0 {
                        None
                    } else {
                        let name = String::from_utf16_lossy(&buffer[..len as usize]);
                        Some(
                            name.trim_end_matches(".exe")
                                .trim_end_matches(".EXE")
                                .to_string(),
                        )
                    };
                    // プロセスハンドルを閉じる
                    let _ = CloseHandle(handle);
                    result
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

    /// 保存されたHWNDのウィンドウにフォーカスを戻す
    #[allow(unsafe_code)]
    pub fn restore_focus_to_last_window() {
        if let Ok(last_hwnd) = LAST_ACTIVE_HWND.lock() {
            if let Some(hwnd_val) = *last_hwnd {
                unsafe {
                    let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
                    let _ = SetForegroundWindow(hwnd);
                }
            }
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
    /// macOS: ダミー実装
    pub fn restore_focus_to_last_window() {}
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod active_window {
    use super::ActiveWindowInfo;
    /// その他のOS: ダミー実装
    pub fn get_active_window_info() -> Option<ActiveWindowInfo> {
        None
    }
    /// その他のOS: ダミー実装
    pub fn restore_focus_to_last_window() {}
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
    if let Some(window) = app.get_window("search") {
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
    if let Some(window) = app.get_window("search") {
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

// ショートカット一覧を取得するコマンド
#[tauri::command]
fn get_shortcuts() -> Vec<NormalizedShortcut> {
    let config = load_keybindings_config();

    config
        .into_iter()
        // 現在のプラットフォームで有効なアプリのみ
        .filter(|app| app.is_available())
        .flat_map(|app| {
            let app_name = app.get_name();
            let app_icon = app.get_icon();
            app.keybindings.into_iter().filter_map(move |kb| {
                // キーが"-"の場合は対象外
                if kb.key == "-" {
                    return None;
                }
                // 表示用に正規化（スペースあり形式に統一）
                let key = normalize_key_for_display(&kb.key);
                // 順次入力キーの区切り文字を変換: "->" → "→"
                let key = key.replace(" -> ", " → ");
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

/// ファイルをシステムのデフォルトアプリケーションで開く
fn open_file_with_default_app(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // explorer.exe でファイルを開く（パスのエスケープが不要）
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("ファイルを開けませんでした: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("ファイルを開けませんでした: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("ファイルを開けませんでした: {e}"))?;
    }

    Ok(())
}

// キーバインド設定ファイルを開くコマンド
#[tauri::command]
fn open_config_file() -> Result<(), String> {
    let path = get_keybindings_config_path().ok_or("設定ファイルのパスが見つかりません")?;
    open_file_with_default_app(&path)
}

// settings.jsonファイルを開くコマンド
#[tauri::command]
fn open_settings_file() -> Result<(), String> {
    let path = get_settings_path().ok_or("設定ファイルのパスが見つかりません")?;

    // ファイルが存在しない場合は作成
    if !path.exists() {
        let settings = AppSettings::default();
        save_settings(&settings)?;
    }

    open_file_with_default_app(&path)
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

// オーバーレイ表示用のペイロード
#[derive(Clone, Serialize)]
struct OverlayPayload {
    app_name: String,
    action_name: String,
    shortcut_key: String,
    duration: u32,
    theme: String,
}

/// オーバーレイウィンドウの幅を計算
fn calculate_overlay_width(shortcut_key: &str) -> f64 {
    const BASE_WIDTH: f64 = 150.0;
    const MODIFIER_WIDTH: f64 = 50.0;
    const SEPARATOR_WIDTH: f64 = 20.0;
    const SEQUENCE_SEPARATOR_WIDTH: f64 = 30.0;
    const DEFAULT_KEY_WIDTH: f64 = 30.0;
    const MIN_WIDTH: f64 = 200.0;
    const MAX_WIDTH: f64 = 600.0;

    let mut width = BASE_WIDTH;

    // 順次入力キーの場合、各ステップを分割して計算
    let steps: Vec<&str> = shortcut_key.split(" → ").collect();
    let is_sequence = steps.len() > 1;

    for step in &steps {
        let step_lower = step.to_lowercase();

        // 修飾キーの幅を加算（各ステップごとにカウント）
        if step_lower.contains("ctrl") || step_lower.contains("control") {
            width += MODIFIER_WIDTH;
        }
        if step_lower.contains("shift") {
            width += MODIFIER_WIDTH;
        }
        if step_lower.contains("alt") || step_lower.contains("option") {
            width += MODIFIER_WIDTH;
        }
        if step_lower.contains("win")
            || step_lower.contains("command")
            || step_lower.contains("cmd")
            || step.contains('⌘')
        {
            width += MODIFIER_WIDTH;
        }

        // 同時押し区切り文字の幅を加算
        let separator_count = step.matches('+').count();
        width += (separator_count as f64) * SEPARATOR_WIDTH;

        // キー自体の幅を加算
        width += DEFAULT_KEY_WIDTH;
    }

    // 順次入力の区切り文字（→）の幅を加算
    if is_sequence {
        width += ((steps.len() - 1) as f64) * SEQUENCE_SEPARATOR_WIDTH;
    }

    // 最小・最大幅でクランプ
    width.clamp(MIN_WIDTH, MAX_WIDTH)
}

/// Windowsでフォーカスを奪わずにウィンドウを表示
#[cfg(target_os = "windows")]
fn show_window_no_focus(window: &tauri::Window) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SW_SHOWNOACTIVATE,
    };

    // HWNDを取得
    if let Ok(hwnd) = window.hwnd() {
        let hwnd = windows::Win32::Foundation::HWND(hwnd.0 as _);

        // SAFETY: Windows APIの呼び出し
        unsafe {
            // SW_SHOWNOACTIVATEでフォーカスを奪わずに表示
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

            // 常に最前面に配置（フォーカスは奪わない）
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn show_window_no_focus(window: &tauri::Window) {
    let _ = window.show();
}

// オーバーレイウィンドウを表示
#[tauri::command]
fn show_overlay(
    app: AppHandle,
    app_name: String,
    action_name: String,
    shortcut_key: String,
) -> Result<(), String> {
    let settings = load_settings();
    let duration = settings.overlay_duration;
    let theme = match settings.theme {
        ThemeSetting::Light => "light".to_string(),
        ThemeSetting::Dark => "dark".to_string(),
        ThemeSetting::System => "system".to_string(),
    };

    // メインウィンドウを非表示
    if let Some(main_window) = app.get_window("search") {
        WINDOW_VISIBLE.store(false, Ordering::SeqCst);
        let _ = main_window.hide();
    }

    // オーバーレイウィンドウを表示（フォーカスは設定しない）
    if let Some(overlay_window) = app.get_window("keyguide") {
        // ウィンドウ幅を計算して設定
        let width = calculate_overlay_width(&shortcut_key);
        let _ = overlay_window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width,
            height: 150.0,
        }));

        // 保存された位置があればその位置に、なければ中央に表示
        if let (Some(x), Some(y)) = (settings.overlay_position.x, settings.overlay_position.y) {
            let _ = overlay_window
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
        } else {
            let _ = overlay_window.center();
        }

        // フォーカスを奪わずに表示
        show_window_no_focus(&overlay_window);

        // 元のアプリにフォーカスを戻す
        active_window::restore_focus_to_last_window();

        // オーバーレイにデータを送信
        let _ = overlay_window.emit(
            "overlay-show",
            OverlayPayload {
                app_name,
                action_name,
                shortcut_key,
                duration,
                theme,
            },
        );

        // Rust側でタイマーを管理（フォーカスがなくてもタイマーが動作するように）
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(u64::from(duration)));
            if let Some(overlay) = app.get_window("keyguide") {
                // Windows API で直接非表示にする（Tauriのhide()が効かない場合の対策）
                #[cfg(target_os = "windows")]
                {
                    if let Ok(hwnd) = overlay.hwnd() {
                        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
                        unsafe {
                            let hwnd = windows::Win32::Foundation::HWND(hwnd.0 as _);
                            let _ = ShowWindow(hwnd, SW_HIDE);
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = overlay.hide();
                }
            }
        });
    }

    Ok(())
}

// オーバーレイウィンドウを非表示
#[tauri::command]
fn hide_overlay(app: AppHandle) {
    if let Some(overlay_window) = app.get_window("keyguide") {
        let _ = overlay_window.hide();
    }
}

// キーバインド設定を生データで取得（設定画面用）
#[tauri::command]
fn get_keybindings_raw() -> Vec<AppConfig> {
    load_keybindings_config()
}

// キーバインド設定を保存（設定画面用）
#[tauri::command]
fn save_keybindings(config: Vec<AppConfig>) -> Result<(), String> {
    save_keybindings_config(&config)
}

// キーバインド設定をデフォルトに戻す
#[tauri::command]
fn reset_keybindings() -> Result<Vec<AppConfig>, String> {
    let defaults = get_default_keybindings();
    save_keybindings_config(&defaults)?;
    Ok(defaults)
}

// キーバインド設定ウィンドウを開く
#[tauri::command]
fn open_keybindings_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_window("keybindings") {
        // ウィンドウを中央に配置して表示
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
        Ok(())
    } else {
        Err("キーバインド設定ウィンドウが見つかりません".to_string())
    }
}

// キーバインド設定ウィンドウを閉じる（非表示にする）
#[tauri::command]
fn close_keybindings_window(app: AppHandle) {
    if let Some(window) = app.get_window("keybindings") {
        let _ = window.hide();
    }
}

// オーバーレイの位置を保存
#[tauri::command]
fn save_overlay_position(x: i32, y: i32) -> Result<(), String> {
    let mut settings = load_settings();
    settings.overlay_position = OverlayPosition {
        x: Some(x),
        y: Some(y),
    };
    save_settings(&settings)
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

            // 設定からホットキーを読み込み（スペースあり/なし両方対応）
            let settings = load_settings();
            let hotkey = normalize_hotkey_for_tauri(&settings.hotkey);

            // グローバルホットキーを登録
            let app_handle_clone = app_handle.clone();
            if let Err(e) = app.global_shortcut_manager().register(&hotkey, move || {
                toggle_window(&app_handle_clone);
            }) {
                eprintln!("Warning: Failed to register global hotkey ({hotkey}): {e:?}");
            }

            // 初期表示
            if let Some(window) = app.get_window("search") {
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
            // 検索ウィンドウのみ処理（キーガイドウィンドウは除外）
            if event.window().label() != "search" {
                return;
            }

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
            get_matched_apps,
            get_shortcuts,
            open_config_file,
            open_settings_file,
            open_keybindings_window,
            close_keybindings_window,
            get_theme_setting,
            set_theme_setting,
            get_system_theme,
            show_overlay,
            hide_overlay,
            save_overlay_position,
            get_keybindings_raw,
            save_keybindings,
            reset_keybindings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
