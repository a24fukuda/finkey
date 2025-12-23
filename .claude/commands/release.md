---
description: バージョン更新、CHANGELOG更新、PR作成、mainへのスカッシュマージを実行
argument-hint: <new-version>
allowed-tools: Read, Edit, Bash(git:*), Bash(gh:*)
---

## リリース準備コマンド

新しいバージョン: `$1`

## 実行手順

以下の手順を順番に実行してください：

### 1. 現在の状態を確認
- `git status` で未コミットの変更がないことを確認
- 変更がある場合は処理を中止し、ユーザーに通知

### 2. CHANGELOGエントリの内容を決定
- `git log` で前回リリース以降のコミット履歴を確認
- feat/fix/chore などのコミットからCHANGELOGに記載すべき内容を抽出
- Added/Changed/Fixed/Removed などのカテゴリに分類

### 3. バージョン更新用ブランチを作成
- ブランチ名: `chore/bump-version-$1`

### 4. バージョンを更新
以下のファイルのバージョンを `$1` に更新：
- `src-tauri/Cargo.toml` の `version` フィールド
- `package.json` の `version` フィールド

### 5. CHANGELOG.md を更新
- 最新のセクションの前に新しいバージョンのエントリを追加
- 日付は本日の日付（YYYY-MM-DD形式）
- 手順2で決定した内容を記載
- ファイル末尾のバージョンリンクも追加

### 6. コミット & プッシュ
```
git add -A
git commit -m "chore: bump version to $1"
git push -u origin chore/bump-version-$1
```

### 7. PR作成
`gh pr create` でPRを作成：
- タイトル: `chore: bump version to $1`
- 本文: 変更内容のサマリー

### 8. スカッシュマージ
`gh pr merge --squash --delete-branch` でmainにマージ

### 9. mainブランチに戻る
```
git checkout main
git pull
```

### 10. 結果を報告
- 作成されたPRのURL
- マージ後のコミットハッシュ
- 更新後のコミット履歴（直近5件）
