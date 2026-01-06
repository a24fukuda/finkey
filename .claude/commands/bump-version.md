---
description: バージョン番号とCHANGELOGを更新
argument-hint: <new-version>
allowed-tools: Read, Edit, Bash(git:*), Bash(gh:*)
---

## バージョン更新コマンド

新しいバージョン: `$1`

## 実行手順

### 1. ブランチを確認

1. `git branch --show-current` で現在のブランチを取得
2. mainブランチの場合:
   - `git checkout -b chore/bump-version-$1` で作業ブランチを作成
3. 作業ブランチの場合:
   - そのまま継続

### 2. CHANGELOGエントリの内容を決定

- `git log` で前回リリース以降のコミット履歴を確認
- [コミットメッセージ規約](../../docs/git/commit.md) を参照し、タイプからCHANGELOGに記載すべき内容を抽出
- Added/Changed/Fixed/Removed などのカテゴリに分類

### 3. バージョンを更新

以下のファイルのバージョンを `$1` に更新:

- `src-tauri/Cargo.toml` の `version` フィールド
- `package.json` の `version` フィールド

### 4. CHANGELOG.md を更新

- 最新のセクションの前に新しいバージョンのエントリを追加
- 日付は本日の日付（YYYY-MM-DD形式）
- 手順2で決定した内容を記載
- ファイル末尾のバージョンリンクも追加

### 5. コミット

```
git add -A
git commit -m "chore: bump version to $1"
```

### 6. PRを作成またはプッシュ

1. `gh pr view --json state` で現在のブランチにPRが存在するか確認
2. PRが存在しない場合:
   - `git push -u origin <branch>`
   - `gh pr create` でPRを作成
3. PRが存在する場合:
   - `git push` のみ実行（PRは自動更新）

### 7. 結果を報告

- 更新したファイルと変更内容
- 作成または更新したPRのURL
- 次のステップ（CI確認 → マージ → release-tag.ps1）
