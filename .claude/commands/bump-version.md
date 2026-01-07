---
description: バージョン更新からリリースまでを状況に応じて実行
argument-hint: <new-version>
allowed-tools: Bash(pwsh:*)
---

# バージョン更新

`./scripts/bump-version.ps1 $1` を実行する。

スクリプトが以下を自動で行う:
- バージョン更新（package.json, Cargo.toml）
- CHANGELOG生成
- コミット・プッシュ
- Pull Request作成
