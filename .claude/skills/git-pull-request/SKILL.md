---
name: git-pull-request
description: |
    PR規約に従ってPull Requestを作成する
    pull request, PR, git pr, プルリクエスト
    PRについて言及している場合に使用する
allowed-tools: [Read, Bash]
---

# Git Pull Request

PR規約に従ってPull Requestを作成する。

## ワークフロー

このチェックリストをコピーして進行状況を追跡する：

```
タスク進捗：
- [ ] ステップ1: 規約を確認する
- [ ] ステップ2: 変更内容を確認する
- [ ] ステップ3: PRを作成する
```

**ステップ1: 規約を確認する**

[docs/git/pull-request.md](../../../docs/git/pull-request.md) を読み、PR規約を確認する。

**ステップ2: 変更内容を確認する**

実行:
- `git status` でブランチと変更状態を確認
- `git log` でコミット履歴を確認
- `git diff main...HEAD` で差分を確認

**ステップ3: PRを作成する**

実行: `gh pr create --title "<type>: <subject>" --body "..."`

## 規約概要

| 項目 | 規則 |
|------|------|
| タイトル形式 | `<type>: <subject>` |
| 本文構成 | Summary + Test plan |
| Summary | 変更内容の概要（箇条書き1-3点） |
| Test plan | レビュアーが再現できる具体的な手順 |

## テンプレート

```markdown
## Summary
- 変更点1
- 変更点2

## Test plan
- [ ] 確認項目1
- [ ] 確認項目2
```

詳細は [docs/git/pull-request.md](../../../docs/git/pull-request.md) を参照。
