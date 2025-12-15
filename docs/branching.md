# ブランチ戦略

本プロジェクトは[GitHub Flow](https://docs.github.com/get-started/using-github/github-flow)を採用する。

## 概要

GitHub Flowは`main`ブランチを中心としたシンプルな戦略である。

- `main`は常にデプロイ可能な状態を維持
- 全ての変更はブランチを切ってPRを経由

```
main ───●─────●──────●──
        ↑     ↑      │
feature─●─────●      │
                     │
fix──────────────────●
```

## ブランチ構成

| ブランチ | 用途 | 寿命 |
|----------|------|------|
| `main` | 本番リリース可能なコード | 永続 |
| `feature/*` | 新機能開発 | 短命 |
| `fix/*` | バグ修正 | 短命 |

## ワークフロー

1. `main`から新しいブランチを作成
2. 変更を加え、意味のある単位でコミット
3. リモートにプッシュし、PRを作成
4. レビューを受け、必要に応じて修正
5. 承認後、`main`へマージしブランチを削除

## ブランチ命名規則

```
<type>/<short-description>
```

| プレフィックス | 用途 | 例 |
|---------------|------|-----|
| `feature/` | 新機能 | `feature/add-export` |
| `fix/` | バグ修正 | `fix/overlay-flicker` |
| `docs/` | ドキュメント | `docs/update-readme` |
| `refactor/` | リファクタリング | `refactor/cleanup-utils` |

**命名のポイント:**
- 英小文字とハイフンを使用
- 短く目的を明確に（2-4単語）
- Issue番号を含めても良い: `fix/123-null-pointer`

## コミットメッセージ

```
<type>: <subject>
```

| タイプ | 用途 |
|--------|------|
| `feat` | 新機能 |
| `fix` | バグ修正 |
| `docs` | ドキュメント |
| `refactor` | リファクタリング |
| `test` | テスト |
| `chore` | 雑務（依存関係更新等） |

**例:**
- `feat: ダークモード切り替え機能を追加`
- `fix: オーバーレイのちらつきを修正`
- `docs: READMEにインストール手順を追加`

## 参考

- [GitHub Flow - GitHub Docs](https://docs.github.com/get-started/using-github/github-flow)
