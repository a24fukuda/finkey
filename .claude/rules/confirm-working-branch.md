# ブランチ確認

プロジェクトファイルを編集する前に現在のブランチを確認し、mainブランチでの意図しない作業を防止する。

## 対象操作

以下の操作を行う前にブランチを確認する:

- ファイルの作成（Write）
- ファイルの編集（Edit）
- ファイルの削除（Bash rm）

## 確認手順

1. `git branch --show-current` で現在のブランチを取得
2. mainブランチの場合、ユーザーに確認:
   - 作業ブランチを作成する
   - mainで作業を継続する

## 確認メッセージ

mainブランチの場合、以下の形式で確認する:

```
現在mainブランチです。作業ブランチを作成しますか？

| 選択肢 | 説明 |
|--------|------|
| 作成する | 新しいブランチを作成してチェックアウト |
| mainで継続 | そのままmainブランチで作業 |
```

## ブランチ名

作業内容に応じた命名規則:

| タイプ | 形式 | 例 |
|--------|------|-----|
| 機能追加 | `feat/<name>` | `feat/dark-mode` |
| バグ修正 | `fix/<name>` | `fix/overlay-flicker` |
| リファクタリング | `refactor/<name>` | `refactor/release-workflow` |
| ドキュメント | `docs/<name>` | `docs/api-guide` |
| バージョン更新 | `chore/bump-version-<ver>` | `chore/bump-version-0.4.9` |