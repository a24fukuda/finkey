---
description: バージョン番号とCHANGELOGを更新
argument-hint: <new-version>
allowed-tools: Read, Edit, Bash(git log:*)
---

## バージョン更新コマンド

新しいバージョン: `$1`

## 実行手順

### 1. CHANGELOGエントリの内容を決定

- `git log` で前回リリース以降のコミット履歴を確認
- [コミットメッセージ規約](../../docs/git/commit.md) を参照し、タイプからCHANGELOGに記載すべき内容を抽出
- Added/Changed/Fixed/Removed などのカテゴリに分類

### 2. バージョンを更新

以下のファイルのバージョンを `$1` に更新:

- `src-tauri/Cargo.toml` の `version` フィールド
- `package.json` の `version` フィールド

### 3. CHANGELOG.md を更新

- 最新のセクションの前に新しいバージョンのエントリを追加
- 日付は本日の日付（YYYY-MM-DD形式）
- 手順1で決定した内容を記載
- ファイル末尾のバージョンリンクも追加

### 4. 結果を報告

更新したファイルと変更内容を報告する。
