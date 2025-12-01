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

// ショートカットの構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub id: u32,
    pub category: String,
    pub action: String,
    pub mac: String,
    pub windows: String,
    pub description: String,
    pub tags: Vec<String>,
}

// 設定ファイルの構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub shortcuts: Vec<Shortcut>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shortcuts: get_default_shortcuts(),
        }
    }
}

// 設定ファイルのパスを取得
fn get_config_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_config_dir = config_dir.join("shortcut-finder");
    Some(app_config_dir.join("shortcuts.json"))
}

// 設定ファイルを読み込む
fn load_config() -> Config {
    if let Some(path) = get_config_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<Config>(&content) {
                    return config;
                }
            }
        }
    }
    // ファイルがなければデフォルトを返し、設定ファイルを作成
    let config = Config::default();
    let _ = save_config(&config);
    config
}

// 設定ファイルを保存
fn save_config(config: &Config) -> Result<(), String> {
    let path = get_config_path().ok_or("設定ディレクトリが見つかりません")?;

    // ディレクトリを作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("ディレクトリ作成エラー: {e}"))?;
    }

    let json = serde_json::to_string_pretty(config).map_err(|e| format!("JSON変換エラー: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("ファイル書き込みエラー: {e}"))?;

    Ok(())
}

// デフォルトのショートカットデータ
#[allow(clippy::too_many_lines)]
fn get_default_shortcuts() -> Vec<Shortcut> {
    vec![
        // 一般的なショートカット
        Shortcut {
            id: 1,
            category: "一般".into(),
            action: "コピー".into(),
            mac: "⌘ + C".into(),
            windows: "Ctrl + C".into(),
            description: "選択したテキストやファイルをクリップボードにコピー".into(),
            tags: vec!["コピー".into(), "copy".into(), "クリップボード".into()],
        },
        Shortcut {
            id: 2,
            category: "一般".into(),
            action: "ペースト（貼り付け）".into(),
            mac: "⌘ + V".into(),
            windows: "Ctrl + V".into(),
            description: "クリップボードの内容を貼り付け".into(),
            tags: vec![
                "ペースト".into(),
                "貼り付け".into(),
                "paste".into(),
                "クリップボード".into(),
            ],
        },
        Shortcut {
            id: 3,
            category: "一般".into(),
            action: "切り取り".into(),
            mac: "⌘ + X".into(),
            windows: "Ctrl + X".into(),
            description: "選択したテキストやファイルを切り取り".into(),
            tags: vec!["切り取り".into(), "cut".into(), "カット".into()],
        },
        Shortcut {
            id: 4,
            category: "一般".into(),
            action: "元に戻す".into(),
            mac: "⌘ + Z".into(),
            windows: "Ctrl + Z".into(),
            description: "直前の操作を取り消し".into(),
            tags: vec![
                "元に戻す".into(),
                "undo".into(),
                "取り消し".into(),
                "アンドゥ".into(),
            ],
        },
        Shortcut {
            id: 5,
            category: "一般".into(),
            action: "やり直し".into(),
            mac: "⌘ + Shift + Z".into(),
            windows: "Ctrl + Y".into(),
            description: "元に戻した操作をやり直し".into(),
            tags: vec!["やり直し".into(), "redo".into(), "リドゥ".into()],
        },
        Shortcut {
            id: 6,
            category: "一般".into(),
            action: "すべて選択".into(),
            mac: "⌘ + A".into(),
            windows: "Ctrl + A".into(),
            description: "すべての項目を選択".into(),
            tags: vec!["すべて選択".into(), "全選択".into(), "select all".into()],
        },
        Shortcut {
            id: 7,
            category: "一般".into(),
            action: "検索".into(),
            mac: "⌘ + F".into(),
            windows: "Ctrl + F".into(),
            description: "ページ内検索を開く".into(),
            tags: vec!["検索".into(), "find".into(), "search".into(), "探す".into()],
        },
        Shortcut {
            id: 8,
            category: "一般".into(),
            action: "保存".into(),
            mac: "⌘ + S".into(),
            windows: "Ctrl + S".into(),
            description: "現在のドキュメントを保存".into(),
            tags: vec!["保存".into(), "save".into(), "セーブ".into()],
        },
        Shortcut {
            id: 9,
            category: "一般".into(),
            action: "印刷".into(),
            mac: "⌘ + P".into(),
            windows: "Ctrl + P".into(),
            description: "印刷ダイアログを開く".into(),
            tags: vec!["印刷".into(), "print".into(), "プリント".into()],
        },
        Shortcut {
            id: 10,
            category: "一般".into(),
            action: "新規作成".into(),
            mac: "⌘ + N".into(),
            windows: "Ctrl + N".into(),
            description: "新しいドキュメントやウィンドウを作成".into(),
            tags: vec!["新規".into(), "new".into(), "作成".into()],
        },
        Shortcut {
            id: 11,
            category: "一般".into(),
            action: "開く".into(),
            mac: "⌘ + O".into(),
            windows: "Ctrl + O".into(),
            description: "ファイルを開くダイアログを表示".into(),
            tags: vec!["開く".into(), "open".into(), "ファイル".into()],
        },
        Shortcut {
            id: 12,
            category: "一般".into(),
            action: "閉じる".into(),
            mac: "⌘ + W".into(),
            windows: "Ctrl + W".into(),
            description: "現在のウィンドウやタブを閉じる".into(),
            tags: vec!["閉じる".into(), "close".into(), "クローズ".into()],
        },
        Shortcut {
            id: 13,
            category: "一般".into(),
            action: "終了".into(),
            mac: "⌘ + Q".into(),
            windows: "Alt + F4".into(),
            description: "アプリケーションを終了".into(),
            tags: vec!["終了".into(), "quit".into(), "exit".into(), "閉じる".into()],
        },
        // テキスト編集
        Shortcut {
            id: 14,
            category: "テキスト編集".into(),
            action: "太字".into(),
            mac: "⌘ + B".into(),
            windows: "Ctrl + B".into(),
            description: "選択したテキストを太字に".into(),
            tags: vec!["太字".into(), "bold".into(), "ボールド".into()],
        },
        Shortcut {
            id: 15,
            category: "テキスト編集".into(),
            action: "斜体".into(),
            mac: "⌘ + I".into(),
            windows: "Ctrl + I".into(),
            description: "選択したテキストを斜体に".into(),
            tags: vec!["斜体".into(), "italic".into(), "イタリック".into()],
        },
        Shortcut {
            id: 16,
            category: "テキスト編集".into(),
            action: "下線".into(),
            mac: "⌘ + U".into(),
            windows: "Ctrl + U".into(),
            description: "選択したテキストに下線を追加".into(),
            tags: vec!["下線".into(), "underline".into(), "アンダーライン".into()],
        },
        Shortcut {
            id: 17,
            category: "テキスト編集".into(),
            action: "行の先頭へ移動".into(),
            mac: "⌘ + ←".into(),
            windows: "Home".into(),
            description: "カーソルを行の先頭に移動".into(),
            tags: vec!["行頭".into(), "先頭".into(), "home".into(), "移動".into()],
        },
        Shortcut {
            id: 18,
            category: "テキスト編集".into(),
            action: "行の末尾へ移動".into(),
            mac: "⌘ + →".into(),
            windows: "End".into(),
            description: "カーソルを行の末尾に移動".into(),
            tags: vec!["行末".into(), "末尾".into(), "end".into(), "移動".into()],
        },
        // ブラウザ
        Shortcut {
            id: 23,
            category: "ブラウザ".into(),
            action: "新しいタブ".into(),
            mac: "⌘ + T".into(),
            windows: "Ctrl + T".into(),
            description: "新しいタブを開く".into(),
            tags: vec![
                "タブ".into(),
                "tab".into(),
                "新規".into(),
                "ブラウザ".into(),
            ],
        },
        Shortcut {
            id: 24,
            category: "ブラウザ".into(),
            action: "タブを閉じる".into(),
            mac: "⌘ + W".into(),
            windows: "Ctrl + W".into(),
            description: "現在のタブを閉じる".into(),
            tags: vec![
                "タブ".into(),
                "tab".into(),
                "閉じる".into(),
                "ブラウザ".into(),
            ],
        },
        Shortcut {
            id: 25,
            category: "ブラウザ".into(),
            action: "閉じたタブを復元".into(),
            mac: "⌘ + Shift + T".into(),
            windows: "Ctrl + Shift + T".into(),
            description: "最後に閉じたタブを再度開く".into(),
            tags: vec![
                "タブ".into(),
                "復元".into(),
                "reopen".into(),
                "ブラウザ".into(),
            ],
        },
        Shortcut {
            id: 28,
            category: "ブラウザ".into(),
            action: "ページを再読み込み".into(),
            mac: "⌘ + R".into(),
            windows: "Ctrl + R / F5".into(),
            description: "現在のページを再読み込み".into(),
            tags: vec![
                "リロード".into(),
                "reload".into(),
                "更新".into(),
                "refresh".into(),
                "ブラウザ".into(),
            ],
        },
        Shortcut {
            id: 30,
            category: "ブラウザ".into(),
            action: "アドレスバーにフォーカス".into(),
            mac: "⌘ + L".into(),
            windows: "Ctrl + L / F6".into(),
            description: "アドレスバーを選択".into(),
            tags: vec![
                "アドレス".into(),
                "URL".into(),
                "ブラウザ".into(),
                "フォーカス".into(),
            ],
        },
        Shortcut {
            id: 32,
            category: "ブラウザ".into(),
            action: "開発者ツール".into(),
            mac: "⌘ + Option + I".into(),
            windows: "F12 / Ctrl + Shift + I".into(),
            description: "開発者ツールを開く".into(),
            tags: vec![
                "開発者ツール".into(),
                "devtools".into(),
                "inspect".into(),
                "デバッグ".into(),
                "ブラウザ".into(),
            ],
        },
        // システム（Mac）
        Shortcut {
            id: 38,
            category: "システム（Mac）".into(),
            action: "Spotlight検索".into(),
            mac: "⌘ + Space".into(),
            windows: "-".into(),
            description: "Spotlight検索を開く".into(),
            tags: vec![
                "spotlight".into(),
                "検索".into(),
                "search".into(),
                "mac".into(),
            ],
        },
        Shortcut {
            id: 39,
            category: "システム（Mac）".into(),
            action: "スクリーンショット（全画面）".into(),
            mac: "⌘ + Shift + 3".into(),
            windows: "Print Screen".into(),
            description: "画面全体のスクリーンショットを撮影".into(),
            tags: vec![
                "スクリーンショット".into(),
                "screenshot".into(),
                "画面キャプチャ".into(),
            ],
        },
        Shortcut {
            id: 40,
            category: "システム（Mac）".into(),
            action: "スクリーンショット（範囲選択）".into(),
            mac: "⌘ + Shift + 4".into(),
            windows: "Win + Shift + S".into(),
            description: "選択範囲のスクリーンショットを撮影".into(),
            tags: vec![
                "スクリーンショット".into(),
                "screenshot".into(),
                "画面キャプチャ".into(),
                "範囲".into(),
            ],
        },
        Shortcut {
            id: 44,
            category: "システム（Mac）".into(),
            action: "アプリの切り替え".into(),
            mac: "⌘ + Tab".into(),
            windows: "Alt + Tab".into(),
            description: "開いているアプリを切り替え".into(),
            tags: vec![
                "切り替え".into(),
                "switch".into(),
                "アプリ".into(),
                "tab".into(),
            ],
        },
        // システム（Windows）
        Shortcut {
            id: 50,
            category: "システム（Windows）".into(),
            action: "スタートメニュー".into(),
            mac: "-".into(),
            windows: "Win".into(),
            description: "スタートメニューを開く".into(),
            tags: vec![
                "スタート".into(),
                "start".into(),
                "メニュー".into(),
                "windows".into(),
            ],
        },
        Shortcut {
            id: 52,
            category: "システム（Windows）".into(),
            action: "エクスプローラーを開く".into(),
            mac: "-".into(),
            windows: "Win + E".into(),
            description: "ファイルエクスプローラーを開く".into(),
            tags: vec![
                "エクスプローラー".into(),
                "explorer".into(),
                "ファイル".into(),
                "windows".into(),
            ],
        },
        Shortcut {
            id: 55,
            category: "システム（Windows）".into(),
            action: "タスクビュー".into(),
            mac: "-".into(),
            windows: "Win + Tab".into(),
            description: "タスクビューを開く".into(),
            tags: vec![
                "タスクビュー".into(),
                "task view".into(),
                "ウィンドウ".into(),
                "windows".into(),
            ],
        },
        // VS Code
        Shortcut {
            id: 61,
            category: "VS Code".into(),
            action: "コマンドパレット".into(),
            mac: "⌘ + Shift + P".into(),
            windows: "Ctrl + Shift + P".into(),
            description: "コマンドパレットを開く".into(),
            tags: vec![
                "コマンド".into(),
                "command".into(),
                "palette".into(),
                "vscode".into(),
            ],
        },
        Shortcut {
            id: 62,
            category: "VS Code".into(),
            action: "クイックオープン".into(),
            mac: "⌘ + P".into(),
            windows: "Ctrl + P".into(),
            description: "ファイルをすばやく開く".into(),
            tags: vec![
                "ファイル".into(),
                "開く".into(),
                "quick open".into(),
                "vscode".into(),
            ],
        },
        Shortcut {
            id: 66,
            category: "VS Code".into(),
            action: "行を削除".into(),
            mac: "⌘ + Shift + K".into(),
            windows: "Ctrl + Shift + K".into(),
            description: "現在の行を削除".into(),
            tags: vec!["削除".into(), "行".into(), "delete".into(), "vscode".into()],
        },
        Shortcut {
            id: 69,
            category: "VS Code".into(),
            action: "行コメント切り替え".into(),
            mac: "⌘ + /".into(),
            windows: "Ctrl + /".into(),
            description: "行コメントの切り替え".into(),
            tags: vec!["コメント".into(), "comment".into(), "vscode".into()],
        },
        Shortcut {
            id: 71,
            category: "VS Code".into(),
            action: "定義に移動".into(),
            mac: "F12".into(),
            windows: "F12".into(),
            description: "シンボルの定義に移動".into(),
            tags: vec![
                "定義".into(),
                "definition".into(),
                "ジャンプ".into(),
                "vscode".into(),
            ],
        },
        Shortcut {
            id: 67,
            category: "VS Code".into(),
            action: "マルチカーソル".into(),
            mac: "⌘ + Option + ↑/↓".into(),
            windows: "Ctrl + Alt + ↑/↓".into(),
            description: "複数行にカーソルを追加".into(),
            tags: vec![
                "マルチカーソル".into(),
                "multi cursor".into(),
                "vscode".into(),
            ],
        },
        Shortcut {
            id: 68,
            category: "VS Code".into(),
            action: "同じ単語を選択".into(),
            mac: "⌘ + D".into(),
            windows: "Ctrl + D".into(),
            description: "同じ単語の次の出現を選択に追加".into(),
            tags: vec![
                "選択".into(),
                "単語".into(),
                "マルチ".into(),
                "vscode".into(),
            ],
        },
        Shortcut {
            id: 74,
            category: "VS Code".into(),
            action: "サイドバー表示切り替え".into(),
            mac: "⌘ + B".into(),
            windows: "Ctrl + B".into(),
            description: "サイドバーの表示/非表示を切り替え".into(),
            tags: vec!["サイドバー".into(), "sidebar".into(), "vscode".into()],
        },
        Shortcut {
            id: 75,
            category: "VS Code".into(),
            action: "ターミナル表示切り替え".into(),
            mac: "⌘ + `".into(),
            windows: "Ctrl + `".into(),
            description: "統合ターミナルの表示/非表示を切り替え".into(),
            tags: vec!["ターミナル".into(), "terminal".into(), "vscode".into()],
        },
        // ブラウザ追加
        Shortcut {
            id: 26,
            category: "ブラウザ".into(),
            action: "次のタブ".into(),
            mac: "⌘ + Option + →".into(),
            windows: "Ctrl + Tab".into(),
            description: "次のタブに移動".into(),
            tags: vec![
                "タブ".into(),
                "tab".into(),
                "次".into(),
                "移動".into(),
                "ブラウザ".into(),
            ],
        },
        Shortcut {
            id: 27,
            category: "ブラウザ".into(),
            action: "前のタブ".into(),
            mac: "⌘ + Option + ←".into(),
            windows: "Ctrl + Shift + Tab".into(),
            description: "前のタブに移動".into(),
            tags: vec![
                "タブ".into(),
                "tab".into(),
                "前".into(),
                "移動".into(),
                "ブラウザ".into(),
            ],
        },
        Shortcut {
            id: 31,
            category: "ブラウザ".into(),
            action: "ブックマーク追加".into(),
            mac: "⌘ + D".into(),
            windows: "Ctrl + D".into(),
            description: "現在のページをブックマークに追加".into(),
            tags: vec![
                "ブックマーク".into(),
                "bookmark".into(),
                "お気に入り".into(),
                "ブラウザ".into(),
            ],
        },
        Shortcut {
            id: 33,
            category: "ブラウザ".into(),
            action: "戻る".into(),
            mac: "⌘ + [".into(),
            windows: "Alt + ←".into(),
            description: "前のページに戻る".into(),
            tags: vec![
                "戻る".into(),
                "back".into(),
                "ブラウザ".into(),
                "ナビゲーション".into(),
            ],
        },
        Shortcut {
            id: 34,
            category: "ブラウザ".into(),
            action: "進む".into(),
            mac: "⌘ + ]".into(),
            windows: "Alt + →".into(),
            description: "次のページに進む".into(),
            tags: vec![
                "進む".into(),
                "forward".into(),
                "ブラウザ".into(),
                "ナビゲーション".into(),
            ],
        },
        Shortcut {
            id: 37,
            category: "ブラウザ".into(),
            action: "シークレットウィンドウ".into(),
            mac: "⌘ + Shift + N".into(),
            windows: "Ctrl + Shift + N".into(),
            description: "新しいシークレット/プライベートウィンドウを開く".into(),
            tags: vec![
                "シークレット".into(),
                "プライベート".into(),
                "incognito".into(),
                "ブラウザ".into(),
            ],
        },
        // システム（Windows）追加
        Shortcut {
            id: 51,
            category: "システム（Windows）".into(),
            action: "設定を開く".into(),
            mac: "-".into(),
            windows: "Win + I".into(),
            description: "Windowsの設定を開く".into(),
            tags: vec!["設定".into(), "settings".into(), "windows".into()],
        },
        Shortcut {
            id: 56,
            category: "システム（Windows）".into(),
            action: "ウィンドウを左半分にスナップ".into(),
            mac: "-".into(),
            windows: "Win + ←".into(),
            description: "ウィンドウを画面の左半分にスナップ".into(),
            tags: vec![
                "スナップ".into(),
                "snap".into(),
                "ウィンドウ".into(),
                "左".into(),
            ],
        },
        Shortcut {
            id: 57,
            category: "システム（Windows）".into(),
            action: "ウィンドウを右半分にスナップ".into(),
            mac: "-".into(),
            windows: "Win + →".into(),
            description: "ウィンドウを画面の右半分にスナップ".into(),
            tags: vec![
                "スナップ".into(),
                "snap".into(),
                "ウィンドウ".into(),
                "右".into(),
            ],
        },
        Shortcut {
            id: 60,
            category: "システム（Windows）".into(),
            action: "クリップボード履歴".into(),
            mac: "-".into(),
            windows: "Win + V".into(),
            description: "クリップボード履歴を表示".into(),
            tags: vec![
                "クリップボード".into(),
                "clipboard".into(),
                "履歴".into(),
                "windows".into(),
            ],
        },
        // Finder / エクスプローラー
        Shortcut {
            id: 77,
            category: "Finder / エクスプローラー".into(),
            action: "新しいフォルダ".into(),
            mac: "⌘ + Shift + N".into(),
            windows: "Ctrl + Shift + N".into(),
            description: "新しいフォルダを作成".into(),
            tags: vec![
                "フォルダ".into(),
                "folder".into(),
                "新規".into(),
                "作成".into(),
            ],
        },
        Shortcut {
            id: 78,
            category: "Finder / エクスプローラー".into(),
            action: "ファイル名を変更".into(),
            mac: "Enter".into(),
            windows: "F2".into(),
            description: "選択したファイルの名前を変更".into(),
            tags: vec!["名前変更".into(), "rename".into(), "ファイル".into()],
        },
        Shortcut {
            id: 79,
            category: "Finder / エクスプローラー".into(),
            action: "ゴミ箱に移動".into(),
            mac: "⌘ + Delete".into(),
            windows: "Delete".into(),
            description: "選択したファイルをゴミ箱に移動".into(),
            tags: vec!["削除".into(), "delete".into(), "ゴミ箱".into()],
        },
        // Slack
        Shortcut {
            id: 84,
            category: "Slack".into(),
            action: "クイックスイッチャー".into(),
            mac: "⌘ + K".into(),
            windows: "Ctrl + K".into(),
            description: "チャンネルやDMをすばやく切り替え".into(),
            tags: vec![
                "slack".into(),
                "切り替え".into(),
                "チャンネル".into(),
                "検索".into(),
            ],
        },
        Shortcut {
            id: 87,
            category: "Slack".into(),
            action: "未読に移動".into(),
            mac: "⌘ + Shift + A".into(),
            windows: "Ctrl + Shift + A".into(),
            description: "未読メッセージに移動".into(),
            tags: vec!["slack".into(), "未読".into(), "メッセージ".into()],
        },
        // Excel / スプレッドシート
        Shortcut {
            id: 88,
            category: "Excel / スプレッドシート".into(),
            action: "セルの編集".into(),
            mac: "F2".into(),
            windows: "F2".into(),
            description: "選択したセルを編集モードに".into(),
            tags: vec!["excel".into(), "編集".into(), "セル".into()],
        },
        Shortcut {
            id: 92,
            category: "Excel / スプレッドシート".into(),
            action: "値のみ貼り付け".into(),
            mac: "⌘ + Shift + V".into(),
            windows: "Ctrl + Shift + V".into(),
            description: "書式なしで値のみ貼り付け".into(),
            tags: vec![
                "excel".into(),
                "貼り付け".into(),
                "値".into(),
                "ペースト".into(),
            ],
        },
        // ターミナル
        Shortcut {
            id: 95,
            category: "ターミナル".into(),
            action: "コマンドを中断".into(),
            mac: "Control + C".into(),
            windows: "Ctrl + C".into(),
            description: "実行中のコマンドを中断".into(),
            tags: vec![
                "ターミナル".into(),
                "terminal".into(),
                "中断".into(),
                "キャンセル".into(),
            ],
        },
        Shortcut {
            id: 96,
            category: "ターミナル".into(),
            action: "画面をクリア".into(),
            mac: "⌘ + K / Control + L".into(),
            windows: "cls / Ctrl + L".into(),
            description: "ターミナル画面をクリア".into(),
            tags: vec![
                "ターミナル".into(),
                "terminal".into(),
                "クリア".into(),
                "clear".into(),
            ],
        },
        // Zoom
        Shortcut {
            id: 100,
            category: "Zoom".into(),
            action: "ミュート切り替え".into(),
            mac: "⌘ + Shift + A".into(),
            windows: "Alt + A".into(),
            description: "マイクのミュート/ミュート解除".into(),
            tags: vec![
                "zoom".into(),
                "ミュート".into(),
                "mute".into(),
                "マイク".into(),
            ],
        },
        Shortcut {
            id: 101,
            category: "Zoom".into(),
            action: "ビデオ切り替え".into(),
            mac: "⌘ + Shift + V".into(),
            windows: "Alt + V".into(),
            description: "ビデオのオン/オフを切り替え".into(),
            tags: vec![
                "zoom".into(),
                "ビデオ".into(),
                "video".into(),
                "カメラ".into(),
            ],
        },
        Shortcut {
            id: 102,
            category: "Zoom".into(),
            action: "画面共有".into(),
            mac: "⌘ + Shift + S".into(),
            windows: "Alt + S".into(),
            description: "画面共有を開始/停止".into(),
            tags: vec![
                "zoom".into(),
                "画面共有".into(),
                "share".into(),
                "screen".into(),
            ],
        },
        // テキスト編集追加
        Shortcut {
            id: 19,
            category: "テキスト編集".into(),
            action: "単語単位で移動（左）".into(),
            mac: "Option + ←".into(),
            windows: "Ctrl + ←".into(),
            description: "カーソルを単語単位で左に移動".into(),
            tags: vec!["単語".into(), "word".into(), "移動".into(), "左".into()],
        },
        Shortcut {
            id: 20,
            category: "テキスト編集".into(),
            action: "単語単位で移動（右）".into(),
            mac: "Option + →".into(),
            windows: "Ctrl + →".into(),
            description: "カーソルを単語単位で右に移動".into(),
            tags: vec!["単語".into(), "word".into(), "移動".into(), "右".into()],
        },
        // システム（Mac）追加
        Shortcut {
            id: 43,
            category: "システム（Mac）".into(),
            action: "デスクトップを表示".into(),
            mac: "F11 / ⌘ + F3".into(),
            windows: "Win + D".into(),
            description: "デスクトップを表示".into(),
            tags: vec!["デスクトップ".into(), "desktop".into(), "表示".into()],
        },
        Shortcut {
            id: 46,
            category: "システム（Mac）".into(),
            action: "強制終了ダイアログ".into(),
            mac: "⌘ + Option + Esc".into(),
            windows: "Ctrl + Shift + Esc".into(),
            description: "アプリケーションの強制終了ダイアログを開く".into(),
            tags: vec![
                "強制終了".into(),
                "force quit".into(),
                "タスクマネージャー".into(),
            ],
        },
        Shortcut {
            id: 47,
            category: "システム（Mac）".into(),
            action: "画面ロック".into(),
            mac: "⌘ + Control + Q".into(),
            windows: "Win + L".into(),
            description: "画面をロック".into(),
            tags: vec!["ロック".into(), "lock".into(), "画面".into()],
        },
        Shortcut {
            id: 49,
            category: "システム（Mac）".into(),
            action: "絵文字ピッカー".into(),
            mac: "⌘ + Control + Space".into(),
            windows: "Win + .".into(),
            description: "絵文字入力パネルを開く".into(),
            tags: vec!["絵文字".into(), "emoji".into(), "顔文字".into()],
        },
    ]
}

// 前回アクティブだったアプリ名を保持
static LAST_ACTIVE_APP: Mutex<Option<String>> = Mutex::new(None);
// ウィンドウが表示中かどうか
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
mod active_window {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{
        GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    /// アクティブなウィンドウのプロセス名を取得（自分自身を除外）
    #[allow(unsafe_code)]
    pub fn get_active_app_name() -> Option<String> {
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
                // 自分自身がアクティブの場合はNoneを返す
                return None;
            }

            let process_handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                process_id,
            )
            .ok()?;

            let mut buffer = [0u16; 260];
            let len = GetModuleBaseNameW(process_handle, None, &mut buffer);

            if len == 0 {
                return None;
            }

            let name = String::from_utf16_lossy(&buffer[..len as usize]);
            // .exe を除去（大文字小文字両方）
            let name = name
                .trim_end_matches(".exe")
                .trim_end_matches(".EXE")
                .to_string();
            Some(name)
        }
    }
}

#[cfg(target_os = "macos")]
mod active_window {
    /// macOS: ダミー実装
    pub fn get_active_app_name() -> Option<String> {
        None
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod active_window {
    /// その他のOS: ダミー実装
    pub fn get_active_app_name() -> Option<String> {
        None
    }
}

// 前回のアクティブアプリを更新する
fn update_last_active_app() {
    if let Some(app_name) = active_window::get_active_app_name() {
        if let Ok(mut last_app) = LAST_ACTIVE_APP.lock() {
            *last_app = Some(app_name);
        }
    }
}

// 前回のアクティブアプリを取得する
fn get_last_active_app() -> Option<String> {
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

// アクティブなアプリ名を取得するコマンド
#[tauri::command]
fn get_active_app() -> Option<String> {
    active_window::get_active_app_name()
}

// ショートカット一覧を取得するコマンド
#[tauri::command]
fn get_shortcuts() -> Vec<Shortcut> {
    load_config().shortcuts
}

// 設定ファイルのパスを取得するコマンド
#[tauri::command]
fn get_config_file_path() -> Option<String> {
    get_config_path().map(|p| p.to_string_lossy().to_string())
}

// ショートカットを保存するコマンド
#[tauri::command]
fn save_shortcuts(shortcuts: Vec<Shortcut>) -> Result<(), String> {
    let config = Config { shortcuts };
    save_config(&config)
}

// 設定ファイルを開くコマンド
#[tauri::command]
fn open_config_file() -> Result<(), String> {
    let path = get_config_path().ok_or("設定ファイルのパスが見つかりません")?;

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
            get_shortcuts,
            get_config_file_path,
            save_shortcuts,
            open_config_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
