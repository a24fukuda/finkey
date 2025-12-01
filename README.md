# Shortcut Finder (Tauri) 🔍⚡

Mac Spotlight風の超軽量ショートカットキー検索アプリ。Tauri製でネイティブ並みのパフォーマンス。

## ✨ 特徴

| 項目 | Tauri版 | Electron版 |
|------|---------|------------|
| **起動速度** | ~100ms | ~1000ms |
| **メモリ使用量** | ~30MB | ~150MB |
| **バイナリサイズ** | ~8MB | ~180MB |
| **レスポンス** | ネイティブ級 | やや遅延 |

- 🚀 **超高速起動** - ネイティブWebViewを使用
- 🎯 **Spotlight風UI** - macOSライクな美しいインターフェース
- 🔍 **インクリメンタル検索** - 日本語・英語対応
- ⌨️ **グローバルショートカット** - どこからでも即座に起動
- 🖥️ **クロスプラットフォーム** - Windows / Mac / Linux

## 📦 インストール

### 必要要件

- **Node.js** 18以上
- **Rust** 1.70以上
- **システム依存**:
  - **Mac**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools, WebView2
  - **Linux**: `webkit2gtk`, `libayatana-appindicator`

### セットアップ

```bash
# Rustのインストール（まだの場合）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# リポジトリをクローン
git clone https://github.com/your-username/shortcut-finder-tauri.git
cd shortcut-finder-tauri

# 依存関係をインストール
npm install

# 開発モードで起動
npm run dev
```

### ビルド

```bash
# リリースビルド
npm run build

# デバッグビルド
npm run build:debug
```

ビルドされたアプリは `src-tauri/target/release/bundle/` に出力されます。

## 🎮 使い方

### 起動

グローバルショートカットでいつでも呼び出し可能：

| OS | ショートカット |
|----|---------------|
| Mac | `⌘ + Shift + K` |
| Windows / Linux | `Ctrl + Shift + K` |

### 操作

| キー | 動作 |
|------|------|
| 文字入力 | ショートカットを検索 |
| `↑` / `↓` | 結果を選択 |
| `Enter` | 詳細を表示 |
| `Esc` | ウィンドウを閉じる |

### 検索のコツ

```
日本語: コピー, 保存, タブ, スクリーンショット
英語: copy, save, tab, screenshot
アプリ名: vscode, slack, zoom
```

## 📁 プロジェクト構造

```
shortcut-finder-tauri/
├── package.json           # npm scripts
├── src/                   # フロントエンド
│   ├── index.html
│   ├── styles.css
│   ├── app.js
│   └── shortcuts.js       # ショートカットDB (104件)
└── src-tauri/             # Rustバックエンド
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── icons/
    └── src/
        └── main.rs        # グローバルショートカット、ウィンドウ制御
```

## 🔧 カスタマイズ

### ショートカットの追加

`src/shortcuts.js` を編集：

```javascript
{
  id: 999,
  category: "カテゴリ名",
  action: "アクション名",
  mac: "⌘ + K",
  windows: "Ctrl + K",
  description: "説明文",
  tags: ["タグ1", "タグ2"]
}
```

### 起動ショートカットの変更

`src-tauri/src/main.rs` を編集：

```rust
let shortcut = if cfg!(target_os = "macos") {
    "Command+Shift+Space"  // 変更
} else {
    "Ctrl+Shift+Space"     // 変更
};
```

## 🛠️ 技術スタック

- **Tauri 1.6** - 軽量デスクトップアプリフレームワーク
- **Rust** - 高速・安全なバックエンド
- **HTML/CSS/JavaScript** - シンプルなフロントエンド（フレームワークなし）

## 📋 収録ショートカット

- **一般** (13件) - コピー、ペースト、保存など
- **テキスト編集** (9件) - 太字、カーソル移動など
- **ブラウザ** (15件) - タブ操作、開発者ツール
- **システム (Mac)** (12件) - Spotlight、スクリーンショット
- **システム (Windows)** (11件) - スナップ、仮想デスクトップ
- **VS Code** (16件) - コマンドパレット、マルチカーソル
- **Finder/エクスプローラー** (7件) - ファイル操作
- **Slack** (4件) - チャンネル切り替え
- **Excel** (7件) - セル操作、関数
- **ターミナル** (5件) - コマンド履歴
- **Zoom** (5件) - ミュート、画面共有

**合計: 104件**

## 📄 ライセンス

MIT License

## 🤝 貢献

プルリクエスト歓迎です！

1. Fork
2. Create branch (`git checkout -b feature/amazing`)
3. Commit (`git commit -m 'Add feature'`)
4. Push (`git push origin feature/amazing`)
5. Create Pull Request
