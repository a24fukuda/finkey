# Custom Slash Commands プロンプティング規約

カスタムスラッシュコマンドを定義する際の規約を定める。

## Frontmatter

### description

```yaml
description: コマンドの簡潔な説明
```

| 項目 | 規則 |
|------|------|
| 必須 | ○ |
| 最大長 | 100文字程度 |
| 文体 | 体言止めまたは動詞終止形 |

**良い例:**

```yaml
description: バージョン更新、CHANGELOG更新、PR作成を実行
description: 自然言語から対象ファイルを特定しコミットを作成する
```

**悪い例:**

```yaml
# 曖昧
description: ヘルパー

# 冗長
description: このコマンドはバージョンを更新してCHANGELOGも更新してPRも作成します
```

### argument-hint

```yaml
argument-hint: [引数のヒント]
```

| 項目 | 規則 |
|------|------|
| 必須 | 引数がある場合は推奨 |
| 形式 | `[任意]` または `<必須>` |

**例:**

```yaml
# 単一引数
argument-hint: [issue-number]
argument-hint: <new-version>

# 複数引数
argument-hint: [pr-number] [priority]
argument-hint: <source-file> <target-file>
```

### allowed-tools

```yaml
allowed-tools: Read, Edit, Bash(git:*)
```

| 項目 | 規則 |
|------|------|
| 必須 | セキュリティ上推奨 |
| 形式 | カンマ区切り |

**ツール指定パターン:**

| パターン | 説明 | 例 |
|----------|------|-----|
| ツール名のみ | そのツールすべて | `Read`, `Edit` |
| プレフィックス指定 | 特定コマンドのみ | `Bash(git:*)`, `Bash(npm run:*)` |
| 複数指定 | カンマ区切り | `Bash(git add:*), Bash(git commit:*)` |

**例:**

```yaml
# Git操作のみ許可
allowed-tools: Read, Edit, Bash(git:*), Bash(gh:*)

# ビルド・テストのみ許可
allowed-tools: Bash(npm run:*), Bash(npm test:*)
```

### model

```yaml
model: claude-3-5-haiku-20241022
```

| 項目 | 規則 |
|------|------|
| 必須 | × |
| 用途 | コマンド専用のモデル指定 |

軽量タスクにはHaikuを指定してコスト削減が可能。

### disable-model-invocation

```yaml
disable-model-invocation: true
```

| 項目 | 規則 |
|------|------|
| 必須 | × |
| デフォルト | `false` |
| 用途 | Claudeによる自動実行を禁止 |

破壊的操作を含むコマンドに推奨。

## 本文構成

### 基本構造

```markdown
---
description: [説明]
argument-hint: [ヒント]
allowed-tools: [ツール]
---

# タイトル（オプション）

コマンドの目的と概要

## 実行手順

1. ステップ1
2. ステップ2
3. ステップ3

## 注意事項（オプション）

前提条件や制約
```

| 項目 | 規則 |
|------|------|
| 行数上限 | 200行以下 |
| 超過時 | スキルとして実装を検討 |

### 引数の参照

| 変数 | 内容 | 例 |
|------|------|-----|
| `$ARGUMENTS` | すべての引数 | `/cmd foo bar` → `foo bar` |
| `$1` | 1番目の引数 | `/cmd foo bar` → `foo` |
| `$2` | 2番目の引数 | `/cmd foo bar` → `bar` |

**使用例:**

```markdown
---
argument-hint: [issue-number] [priority]
---

イシュー #$1 を修正する。
優先度: $2

すべての引数: $ARGUMENTS
```

### 特殊プレフィックス

| プレフィックス | 用途 | 例 |
|---------------|------|-----|
| `@` | ファイル内容を埋め込み | `@src/main.ts` |
| `!` | Bashコマンド実行結果を埋め込み | `!git status` |

**ファイル参照:**

```markdown
対象ファイル:
@$1

設定ファイル:
@package.json
```

**コマンド実行:**

```markdown
現在のブランチ: !git branch --show-current
変更状況: !git status --short
```

## ファイル構成

```
.claude/
└── commands/
    ├── release.md           # /release (project)
    └── git-commit.md        # /git-commit (project)
```

| 対象 | 規則 | 例 |
|------|------|-----|
| ディレクトリ | 小文字、ハイフン区切り | `commands/` |
| ファイル | 小文字、ハイフン区切り、`.md` | `git-commit.md` |
| コマンド名 | ファイル名から `.md` を除いたもの | `git-commit` |

### サブディレクトリ

サブディレクトリはコマンドのグループ化に使用できる。

```
.claude/
└── commands/
    ├── release.md               # /release (project)
    └── frontend/
        └── component.md         # /component (project:frontend)
```

| 項目 | 説明 |
|------|------|
| コマンド名 | ファイル名のみで決定（サブディレクトリは影響しない） |
| 説明表示 | `(project:サブディレクトリ名)` と表示される |
| 用途 | 関連コマンドのグループ化、可読性向上 |

**注意:** 同名ファイルを複数のサブディレクトリに配置すると競合する。

## 設計原則

基本原則は [agent-skills.md](agent-skills.md) の「設計原則」を参照。

### コマンド固有の注意点

**確認ステップの挿入:**

破壊的操作の前に確認を求める。

```markdown
## 実行手順

### 1. 変更内容を確認
対象ファイルと変更内容をユーザーに提示する。

### 2. 承認を得る
「これでよろしいですか？」と確認する。

### 3. 実行
承認後に処理を実行する。
```

**エラーハンドリング:**

失敗時の対応を明記する。

```markdown
### 1. 現在の状態を確認
- `git status` で未コミットの変更がないことを確認
- 変更がある場合は処理を中止し、ユーザーに通知
```

**冪等性の考慮:**

複数回実行しても問題ない設計にする。

## スキルとの使い分け

| 観点 | コマンド | スキル |
|------|----------|--------|
| ファイル構成 | 単一 `.md` | ディレクトリ + `SKILL.md` |
| 発見方法 | 明示的（`/command`） | 自動（コンテキストベース） |
| 複雑度 | シンプル〜中程度 | 中程度〜複雑 |
| 引数 | `$ARGUMENTS`, `$1`〜 | なし |
| 行数目安 | 200行以下 | 500行以下 |

**コマンドが適切:**

- 引数を受け取る定型タスク
- 明示的に実行するワークフロー
- 単一ファイルで完結する手順

**スキルが適切:**

- 自動検出が必要なケース
- 複数リソースを参照する複雑なワークフロー
- 段階的開示が有効な詳細手順

**併用パターン:**

コマンドからスキルを呼び出す構成も可能。

```markdown
---
description: コミットを作成する
argument-hint: [変更の説明]
---

# Git Commit

引数: $ARGUMENTS

[git-commit スキル](../skills/git-commit/SKILL.md) の手順に従って処理する。
```

## チェックリスト

```
[ ] description が具体的で簡潔
[ ] 引数がある場合 argument-hint を指定
[ ] allowed-tools で必要なツールのみ許可
[ ] 破壊的操作に確認ステップがある
[ ] 失敗時の対応が明記されている
[ ] 200行以下（超過時はスキル化を検討）
[ ] テスト実行で動作確認
```

## 参考

- [Agent Skills 規約](agent-skills.md)
- [公式ドキュメント](https://docs.anthropic.com/en/docs/claude-code/slash-commands)
