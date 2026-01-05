# コミットメッセージ規約

本プロジェクトのコミットメッセージの記述ルールを定める。

## 形式

```
<type>: <subject>

[body]

[footer]
```

- **type**: 変更の種類（必須）
- **subject**: 変更の要約（必須、50文字以内推奨）
- **body**: 詳細説明（任意、72文字で折り返し）
- **footer**: 関連Issue等（任意）

## タイプ一覧

| タイプ | 用途 |
|--------|------|
| `feat` | 新機能 |
| `fix` | バグ修正 |
| `docs` | ドキュメント |
| `refactor` | リファクタリング |
| `test` | テスト |
| `chore` | 雑務（依存関係更新、ビルド設定等） |

## 本文とフッター

本文（body）は変更の「なぜ」を説明する。コードを見れば分かる「何を」は書かない。

```
fix: オーバーレイのちらつきを修正

描画タイミングがずれていたため、
ウィンドウ表示前にレンダリングを完了させるよう変更。

Fixes #42
```

フッター（footer）にはIssue参照を記載できる。

| キーワード | 用途 |
|-----------|------|
| `Fixes #n` | Issueをクローズ |
| `Refs #n` | 関連Issueへの参照 |

## 例

**良い例:**
```
feat: ダークモード切り替え機能を追加
fix: オーバーレイのちらつきを修正
docs: READMEにインストール手順を追加
chore: bump version to 0.4.2
```

**悪い例:**
```
fix bug              # 何を修正したか不明
Update main.rs       # typeがない、内容が曖昧
feat: 追加           # subjectが不明確
```

## 参考

- [Conventional Commits](https://www.conventionalcommits.org/)
- [How to Write a Git Commit Message](https://cbea.ms/git-commit/)
