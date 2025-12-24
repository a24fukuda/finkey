# Agent Skills プロンプティング規約

Agent Skillsを定義する際の規約を定める。

## Frontmatter

### name

```yaml
name: skill-name
```

| 項目 | 規則 |
|------|------|
| 文字種 | 小文字、数字、ハイフンのみ |
| 最大長 | 64文字 |
| 禁止 | XMLタグ、予約語（`anthropic`, `claude`） |

**推奨形式（動名詞形）:**

```
processing-pdfs
analyzing-spreadsheets
managing-databases
testing-code
writing-documentation
```

**許容される代替形式:**

| 形式 | 例 |
|------|-----|
| 名詞句 | `pdf-processing`, `spreadsheet-analysis` |
| アクション指向 | `process-pdfs`, `analyze-spreadsheets` |

**避けるべき名前:**

| 種類 | 例 |
|------|-----|
| 曖昧 | `helper`, `utils`, `tools` |
| 過度に一般的 | `documents`, `data`, `files` |
| 予約語使用 | `anthropic-helper`, `claude-tools` |

### description

```yaml
description: |
    [説明 - 何をするか]
    [キーワード - カンマ区切り]
    [使用条件 - いつ使用するか]
```

| 項目 | 規則 |
|------|------|
| 最大長 | 1024文字 |
| 文体 | 三人称（「処理する」「生成する」） |
| 構成 | 説明 → キーワード → 使用条件（各行で分離） |
| 禁止 | XMLタグ |

**構成要素:**

| 行 | 内容 | 例 |
|----|------|-----|
| 1行目 | 機能説明 | コミット規約に従ってコミットを作成する |
| 2行目 | キーワード | commit, git commit, コミット |
| 3行目 | 使用条件 | コミットについて言及している場合に使用する |

**良い例:**

```yaml
description: |
    コミット規約に従ってコミットを作成する
    commit, git commit, コミット
    コミットについて言及している場合に使用する
```

```yaml
description: |
    Excelスプレッドシートを分析し、ピボットテーブルを作成し、グラフを生成する
    Excel, スプレッドシート, 表形式データ, .xlsx
    Excelファイルの分析について言及している場合に使用する
```

**避けるべき例:**

```yaml
# 曖昧
description: ドキュメントに役立つ

# 一人称/二人称
description: Excelファイルの処理をお手伝いできます
```

### allowed-tools

```yaml
allowed-tools: [Read, Bash, Edit, Write, Glob, Grep]
```

必要最小限のツールのみ許可する。

## 本文構成

### 基本構造

```markdown
# スキル名

導入文（1行で目的を説明）

## セクション1

内容

## セクション2

内容
```

| 項目 | 規則 |
|------|------|
| 行数上限 | 500行以下 |
| 超過時 | 別ファイルに分割し参照 |

### 段階的開示

SKILL.mdは概要として機能し、詳細は別ファイルに配置する。

```
skill-name/
├── SKILL.md              # メイン指示（トリガー時にロード）
├── FORMS.md              # 詳細ガイド（必要時にロード）
├── reference.md          # APIリファレンス（必要時にロード）
└── scripts/
    └── validate.py       # ユーティリティスクリプト
```

**参照の書き方:**

```markdown
## 高度な機能

**フォーム入力**: 完全なガイドについては[FORMS.md](FORMS.md)を参照
**APIリファレンス**: すべてのメソッドについては[REFERENCE.md](REFERENCE.md)を参照
```

**参照の深さ:** 1レベルまで（SKILL.mdから直接リンク）

```markdown
# 悪い例（深すぎる）
SKILL.md → advanced.md → details.md

# 良い例（1レベル）
SKILL.md → advanced.md
SKILL.md → reference.md
SKILL.md → examples.md
```

### 目次

100行以上の参照ファイルには目次を含める。

```markdown
# APIリファレンス

## 目次
- 認証とセットアップ
- コアメソッド（作成、読み取り、更新、削除）
- 高度な機能（バッチ操作、ウェブフック）
- エラー処理パターン

## 認証とセットアップ
...
```

### ワークフロー

複雑な操作は明確なステップに分割する。チェックリストを提供し進行状況を追跡可能にする。

````markdown
## PDFフォーム入力ワークフロー

このチェックリストをコピーして進行状況を追跡する：

```
タスク進捗：
- [ ] ステップ1：フォームを分析する
- [ ] ステップ2：フィールドマッピングを作成する
- [ ] ステップ3：マッピングを検証する
- [ ] ステップ4：フォームに入力する
- [ ] ステップ5：出力を確認する
```

**ステップ1：フォームを分析する**

実行：`python scripts/analyze_form.py input.pdf`
````

### フィードバックループ

バリデータを実行 → エラーを修正 → 繰り返す

```markdown
## ドキュメント編集プロセス

1. `word/document.xml`に編集を加える
2. **すぐに検証する**：`python scripts/validate.py dir/`
3. 検証が失敗した場合：
   - エラーメッセージを確認する
   - 問題を修正する
   - 検証を再度実行する
4. **検証が成功したときのみ続行する**
```

## ファイル構成

```
.claude/skills/
└── skill-name/
    └── SKILL.md
```

| 対象 | 規則 | 例 |
|------|------|-----|
| ディレクトリ | 小文字、ハイフン区切り | `pdf-processing/` |
| ファイル | `SKILL.md` 固定 | `SKILL.md` |
| パス区切り | フォワードスラッシュのみ | `reference/guide.md` |

## 設計原則

### 簡潔さ

Claudeが既に知っていることは書かない。

**良い例（約50トークン）:**

````markdown
## PDFテキストの抽出

テキスト抽出にはpdfplumberを使用する：

```python
import pdfplumber
with pdfplumber.open("file.pdf") as pdf:
    text = pdf.pages[0].extract_text()
```
````

**悪い例（約150トークン）:**

```markdown
## PDFテキストの抽出

PDF（ポータブルドキュメントフォーマット）ファイルは、テキスト、画像、
その他のコンテンツを含む一般的なファイル形式です。PDFからテキストを
抽出するには、ライブラリを使用する必要があります...
```

### 自由度の設定

タスクの性質に応じて指示の具体性を調整する。

| 自由度 | 使用場面 | 指示の形式 |
|--------|----------|------------|
| 高 | 複数アプローチが有効、コンテキスト依存 | テキストベースの指示 |
| 中 | 推奨パターンあり、変動許容 | 疑似コード、パラメータ付きスクリプト |
| 低 | 操作が脆弱、一貫性重要 | 特定のスクリプト、パラメータなし |

### 用語の一貫性

1つの用語を選択し、スキル全体で統一する。

```
# 良い例
常に「APIエンドポイント」
常に「フィールド」

# 悪い例
「APIエンドポイント」「URL」「APIルート」「パス」を混在
```

### 時間依存情報の回避

古くなる情報は含めない。

```markdown
# 悪い例
2025年8月前にこれを行っている場合は、古いAPIを使用してください。

# 良い例
## 現在の方法
v2 APIエンドポイントを使用する：`api.example.com/v2/messages`

<details>
<summary>レガシーv1 API（2025-08で廃止）</summary>
v1 APIは以下を使用していた：`api.example.com/v1/messages`
</details>
```

### 選択肢の制限

必要でない限り複数のアプローチを提示しない。

````markdown
# 悪い例
「pypdfまたはpdfplumberまたはPyMuPDFまたは...を使用できます」

# 良い例
「テキスト抽出にはpdfplumberを使用する：
```python
import pdfplumber
```
スキャンされたPDFでOCRが必要な場合は、代わりにpdf2imageとpytesseractを使用する。」
````

## スクリプト

### エラー処理

Claudeに任せるのではなくエラー条件を処理する。

```python
# 良い例
def process_file(path):
    try:
        with open(path) as f:
            return f.read()
    except FileNotFoundError:
        print(f"ファイル{path}が見つかりません、デフォルトを作成しています")
        with open(path, 'w') as f:
            f.write('')
        return ''

# 悪い例
def process_file(path):
    return open(path).read()  # 失敗してClaudeに任せる
```

### 定数の文書化

マジックナンバーを避け、値の根拠を説明する。

```python
# 良い例
# HTTPリクエストは通常30秒以内に完了する
REQUEST_TIMEOUT = 30

# 悪い例
TIMEOUT = 47  # なぜ47？
```

### 依存関係の明示

パッケージが利用可能と仮定しない。

````markdown
# 悪い例
「pdfライブラリを使用してファイルを処理します。」

# 良い例
「必要なパッケージをインストールする：`pip install pypdf`

```python
from pypdf import PdfReader
reader = PdfReader("file.pdf")
```」
````

### MCPツール参照

完全修飾ツール名を使用する。

```markdown
# 形式
ServerName:tool_name

# 例
BigQuery:bigquery_schemaツールを使用してテーブルスキーマを取得する。
GitHub:create_issueツールを使用して問題を作成する。
```

## テスト

### モデル別テスト

使用予定のすべてのモデルでテストする。

| モデル | 確認事項 |
|--------|----------|
| Haiku | 十分なガイダンスを提供しているか |
| Sonnet | 明確で効率的か |
| Opus | 過度な説明を避けているか |

### 評価駆動開発

1. スキルなしでタスクを実行し、失敗点を特定する
2. 失敗点をテストする評価を作成する
3. ベースライン（スキルなし）のパフォーマンスを測定する
4. 失敗点に対処する最小限の指示を書く
5. 評価を実行し、改善を繰り返す

## チェックリスト

### コア品質

```
[ ] descriptionは具体的で主要な用語を含む
[ ] descriptionは何をするか + いつ使用するかを含む
[ ] SKILL.md本文は500行以下
[ ] 追加の詳細は別ファイルに配置（必要な場合）
[ ] 時間に敏感な情報がない
[ ] 全体で一貫した用語
[ ] ファイル参照は1レベルの深さ
[ ] ワークフローに明確なステップがある
```

### スクリプト

```
[ ] エラー処理が明示的
[ ] マジックナンバーがない（すべての値に根拠）
[ ] 必要なパッケージがリストされている
[ ] Windowsスタイルのパスがない（フォワードスラッシュのみ）
[ ] 重要な操作に検証ステップがある
```

### テスト

```
[ ] 少なくとも3つの評価を作成した
[ ] Haiku、Sonnet、Opusでテストした
[ ] 実際の使用シナリオでテストした
```

## 参考

- [公式ベストプラクティス](https://platform.claude.com/docs/ja/agents-and-tools/agent-skills/best-practices)
- [スキル概要](https://platform.claude.com/docs/ja/agents-and-tools/agent-skills/overview)
