use serde::Deserialize;

// ============================================================
// defaults/settings.json の検証用構造体
// ============================================================

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum ThemeSetting {
    System,
    Light,
    Dark,
}

#[derive(Deserialize)]
struct DefaultSettings {
    #[allow(dead_code)]
    theme: ThemeSetting,
    #[allow(dead_code)]
    hotkey: String,
    #[allow(dead_code)]
    overlay_duration: u32,
}

// ============================================================
// defaults/keybindings.json の検証用構造体
// ============================================================

#[derive(Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum AppBind {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize)]
struct Keybinding {
    #[allow(dead_code)]
    action: String,
    #[allow(dead_code)]
    key: String,
    #[serde(default)]
    #[allow(dead_code)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
enum OsType {
    Windows,
    #[serde(rename = "macos")]
    MacOS,
}

#[derive(Deserialize)]
struct AppConfig {
    #[serde(default)]
    #[allow(dead_code)]
    icon: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    bind: Option<AppBind>,
    #[serde(default)]
    #[allow(dead_code)]
    os: Option<OsType>,
    #[serde(default)]
    #[allow(dead_code)]
    keybindings: Vec<Keybinding>,
}

// ============================================================
// コンパイル時検証
// ============================================================

fn main() {
    // ターゲットOSを取得
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let platform = if target_os == "macos" { "macos" } else { "windows" };

    // プラットフォーム別のdefaults設定を検証
    validate_defaults(platform);

    // 両方のプラットフォームのdefaultsファイルが変更されたら再ビルド
    println!("cargo:rerun-if-changed=defaults/windows/settings.json");
    println!("cargo:rerun-if-changed=defaults/windows/keybindings.json");
    println!("cargo:rerun-if-changed=defaults/macos/settings.json");
    println!("cargo:rerun-if-changed=defaults/macos/keybindings.json");

    tauri_build::build();
}

fn validate_defaults(platform: &str) {
    // settings.json を検証
    let settings_path = format!("defaults/{platform}/settings.json");
    let settings_json = std::fs::read_to_string(&settings_path)
        .unwrap_or_else(|e| panic!("\n\n{settings_path} の読み込みに失敗しました: {e}\n\n"));

    if let Err(e) = serde_json::from_str::<DefaultSettings>(&settings_json) {
        panic!(
            "\n\n========================================\n\
             {settings_path} の検証に失敗しました\n\
             ----------------------------------------\n\
             {e}\n\
             ========================================\n\n"
        );
    }

    // keybindings.json を検証
    let keybindings_path = format!("defaults/{platform}/keybindings.json");
    let keybindings_json = std::fs::read_to_string(&keybindings_path)
        .unwrap_or_else(|e| panic!("\n\n{keybindings_path} の読み込みに失敗しました: {e}\n\n"));

    if let Err(e) = serde_json::from_str::<Vec<AppConfig>>(&keybindings_json) {
        panic!(
            "\n\n========================================\n\
             {keybindings_path} の検証に失敗しました\n\
             ----------------------------------------\n\
             {e}\n\
             ========================================\n\n"
        );
    }
}
