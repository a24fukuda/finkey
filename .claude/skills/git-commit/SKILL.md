---
name: git-commit
description: |
    コミット規約に従ってコミットを作成する
    commit, git commit, コミット, 変更をコミット
    コミットについて言及している場合に使用する
allowed-tools: [Read, Bash]
---

# Git Commit

コミット規約に従ってコミットを作成する。

## ワークフロー

このチェックリストをコピーして進行状況を追跡する：

```
タスク進捗：
- [ ] ステップ1: 規約を確認する
- [ ] ステップ2: 変更内容を確認する
- [ ] ステップ3: コミットを作成する
```

**ステップ1: 規約を確認する**

[docs/git/commit.md](../../../docs/git/commit.md) を読み、コミットメッセージ規約を確認する。

**ステップ2: 変更内容を確認する**

実行: `git status` および `git diff`

**ステップ3: コミットを作成する**

1. 変更をステージングする: `git add <files>`
2. コミットを作成する: `git commit -m "<type>: <subject>"`

## 規約概要

| 項目 | 規則 |
|------|------|
| 形式 | `<type>: <subject>` |
| type | feat, fix, docs, refactor, test, chore |
| subject | 50文字以内、変更内容を要約 |
| body | 必要に応じて「なぜ」を説明 |

詳細は [docs/git/commit.md](../../../docs/git/commit.md) を参照。
