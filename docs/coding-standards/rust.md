# Rustコーディング規約

本プロジェクトにおけるRustコードの記述ルール。

## フォーマット

`cargo fmt`で自動整形する。

```bash
cargo fmt
```

**ルール:**
- インデント: スペース4つ
- 行末の空白: 削除
- 最終行: 改行で終了

## 命名規則

| 対象 | 規則 | 例 |
|------|------|-----|
| 構造体・列挙型・トレイト | PascalCase | `AppConfig`, `ThemeSetting` |
| 関数・メソッド・変数 | snake_case | `get_shortcuts`, `load_settings` |
| 定数・静的変数 | SCREAMING_SNAKE_CASE | `DEFAULT_APP_ICON`, `SETTINGS_CACHE` |
| モジュール | snake_case | `active_window` |
| 型パラメータ | 大文字1文字または短いPascalCase | `T`, `E`, `Item` |

## コード構成

### ファイル構造

```
src-tauri/
  src/
    main.rs      # エントリーポイント
  defaults/      # デフォルト設定ファイル
  build.rs       # ビルドスクリプト
  Cargo.toml     # 依存関係定義
```

### インポート順序

1. 標準ライブラリ（`std::`）
2. 外部クレート
3. 内部モジュール（`crate::`、`super::`）

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
```

### 構造体の定義順序

1. derive属性
2. serde属性
3. フィールド定義
4. implブロック

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct AppSettings {
    #[serde(default)]
    pub theme: ThemeSetting,
}

impl AppSettings {
    // メソッド定義
}
```

## 安全性（unsafe）

### 基本方針

- `unsafe`の使用は最小限に抑える
- 使用する場合は`#[allow(unsafe_code)]`属性と`// SAFETY:`コメントを付与

### 記述例

```rust
#[allow(unsafe_code)]
pub fn get_active_window_info() -> Option<ActiveWindowInfo> {
    // SAFETY: Windows APIの呼び出しに必要
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        // ...
    }
}
```

### プラットフォーム固有コード

`cfg`属性でプラットフォームを分離する。

```rust
#[cfg(target_os = "windows")]
mod active_window {
    // Windows固有の実装
}

#[cfg(target_os = "macos")]
mod active_window {
    // macOS固有の実装
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod active_window {
    // その他のOS用フォールバック
}
```

## エラーハンドリング

### Result型の使用

エラーが発生しうる関数は`Result<T, E>`を返す。

```rust
fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path().ok_or("設定ディレクトリが見つかりません")?;
    // ...
    Ok(())
}
```

### エラーメッセージ

- 日本語で記述
- 原因を明確に示す
- フォーマット文字列ではインライン変数を使用

```rust
// Good
format!("ファイル書き込みエラー: {e}")

// Bad
format!("ファイル書き込みエラー: {}", e)
```

### Option型の扱い

- `unwrap()`は使用しない（テストを除く）
- `unwrap_or_default()`または`?`演算子を使用

```rust
// Good
let name = self.name.clone().unwrap_or_default();

// Bad
let name = self.name.clone().unwrap();
```

## ドキュメンテーション

### ドキュメントコメント

公開APIには`///`でドキュメントを記述する。

```rust
/// ショートカットキー文字列を正規化（Tauri API用）
/// スペースあり/なし両方の入力形式を受け付け、スペースなし形式に変換
fn normalize_hotkey_for_tauri(key: &str) -> String {
    key.replace(" + ", "+")
}
```

### 内部コメント

複雑なロジックや意図が明確でない箇所には`//`でコメントを付与。

```rust
// 次の文字も + の場合はキー自体が + なのでスキップ
if i + 1 < chars.len() && chars[i + 1] == '+' {
    result.push_str(" + +");
    i += 2;
    continue;
}
```

## 依存関係

### Cargo.tomlの記述

- バージョンは明示的に指定
- featuresは必要最小限に限定

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
tauri = { version = "1.8", features = ["shell-open", "system-tray"] }
```

### 依存追加の判断基準

1. 標準ライブラリで代替可能か確認
2. メンテナンス状況を確認
3. ライセンスの互換性を確認

## 品質管理

### 静的解析

`cargo clippy`で警告をチェックする。

```bash
cargo clippy -- -D warnings
```

### ビルド確認

```bash
cargo build --release
```

### チェックリスト

コミット前に以下を確認:

```
[ ] cargo fmt でフォーマット済み
[ ] cargo clippy で警告なし
[ ] cargo build --release が成功
```

## 参考

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tauri Documentation](https://tauri.app/v1/guides/)
