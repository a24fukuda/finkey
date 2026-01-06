---
description: バージョン更新からリリースまでを状況に応じて実行
argument-hint: <new-version>
allowed-tools: Read, Edit, Bash(git:*), Bash(gh pr:*)
---

## バージョン更新コマンド

新しいバージョン: `$1`

## 実行フロー

以下の状態を順に確認し、適切な処理を実行する。

### 1. タグ確認

`git tag -l v$1` でタグの存在を確認。

- **存在する場合**: 警告「v$1 は既にリリース済みです」→ 終了

### 2. PR確認

`gh pr view --json state,url` で現在のブランチのPR状態を確認。

#### PRが存在する場合

`gh pr checks` でCI状態を確認。

**CI成功の場合:**
1. 確認「CIが成功しています。マージしますか？」
2. [Pull Request規約](../../docs/git/pull-request.md) に従ってスカッシュマージ
3. mainブランチに切り替え・pull
4. `./scripts/release-tag.ps1` の実行を案内
5. 終了

**CI実行中の場合:**
1. 報告「CIが実行中です。完了後に再度実行してください」
2. 終了

**CI失敗の場合:**
1. `git status --porcelain` と `git log origin/HEAD..HEAD` で未pushの変更を確認
2. **変更なしの場合:**
   - `gh pr checks` で失敗したジョブを特定
   - 失敗要因を調査・報告
   - 終了
3. **変更ありの場合:**
   - 未コミット差分があればコミット
   - `git push` でCI再実行をトリガー
   - 報告「pushしました。CIの再実行を待ってください」
   - 終了

#### PRが存在しない場合

次のステップへ進む。

### 3. ブランチ確認

`git branch --show-current` で現在のブランチを確認。

- **mainの場合**: `git checkout -b chore/bump-version-$1` でブランチ作成

### 4. 差分確認

`git status --porcelain` で未コミット差分を確認。

- **差分ありの場合**: [コミットメッセージ規約](../../docs/git/commit.md) に従ってコミット

### 5. バージョン確認

`package.json` の現在のバージョンを確認。

- **$1 と異なる場合**:
  - `src-tauri/Cargo.toml` の `version` を更新
  - `package.json` の `version` を更新

### 6. CHANGELOG確認

`CHANGELOG.md` に `## [$1]` エントリが存在するか確認。

- **存在しない場合**:
  - `git log` で前回リリース以降のコミット履歴を確認
  - [コミットメッセージ規約](../../docs/git/commit.md) を参照し、内容を分類
  - CHANGELOG.md にエントリを追加
  - ファイル末尾のバージョンリンクも追加
  - コミット

- **存在するが更新が必要な場合**:
  - 新しいコミットを既存エントリに追記
  - コミット

### 7. PR作成

1. `git push -u origin <branch>`
2. [Pull Request規約](../../docs/git/pull-request.md) に従ってPR作成
3. 報告「PRを作成しました。CIの完了を待ってから再度実行してください」

## 出力形式

各ステップで以下の形式で状態を報告する:

```
=== bump-version 状態レポート ===

バージョン:  0.4.9
ブランチ:    chore/bump-version-0.4.9
PR:          #35 (Open)
CI:          ✓ 成功 / ⏳ 実行中 / ✗ 失敗
タグ:        未作成 / 作成済み

→ [次のアクション]
```
