# 依存先選択基準

パッケージやクレートを依存先として採用する際の安全性評価基準を定める。

## 評価の原則

依存追加の検討時は以下の順序で評価する。

```
1. 必要性の確認 → 本当に必要か、既存機能で代替できないか
2. チェック項目の確認 → メンテナンス・セキュリティ・ライセンス・品質
3. 試験的導入 → 隔離環境でのテスト
```

**原則:** 依存は少ないほど良い。新規依存は攻撃対象領域を拡大する。

## チェック項目

### メンテナンス

| 項目 | 基準 | 必須 |
|------|------|:----:|
| 最終コミット | 12ヶ月以内 | Yes |
| 最終リリース | 12ヶ月以内 | Yes |
| メンテナ数 | 2名以上（理想は複数組織） | Yes |
| バージョン安定性 | `0.x`、`alpha`、`beta` は慎重に判断 | - |

### セキュリティ

| 項目 | 基準 | 必須 |
|------|------|:----:|
| 既知の脆弱性 | Critical/High がないこと | Yes |
| セキュリティポリシー | SECURITY.md または報告手順が存在 | Yes |
| 依存の深さ | 間接依存が過度に多くないこと | Yes |
| unsafe使用（Rust） | 必要最小限、かつ妥当な理由があること | - |

**検査ツール:**

| ツール | 用途 |
|--------|------|
| `cargo audit` | RustSec DB による脆弱性スキャン |
| `cargo deny` | ライセンス・重複・ソース検査 |
| `npm audit` | npm パッケージの脆弱性スキャン |
| [OpenSSF Scorecard](https://securityscorecards.dev/) | プロジェクトのセキュリティスコア |
| [deps.dev](https://deps.dev/) | 依存関係の可視化・脆弱性確認 |

```bash
# Rust
cargo audit
cargo deny check

# npm
npm audit
```

### ライセンス

| 項目 | 基準 | 必須 |
|------|------|:----:|
| ライセンス明記 | 全コンポーネントにライセンスがあること | Yes |
| 互換性 | プロジェクトのライセンスと互換 | Yes |
| 許容ライセンス | MIT, Apache-2.0, BSD 系など | - |

### 品質・コミュニティ

| 項目 | 確認内容 | 必須 |
|------|----------|:----:|
| 利用実績 | ダウンロード数、依存されているパッケージ数 | - |
| テスト | CI が設定され、テストが存在する | - |
| ドキュメント | API ドキュメントが整備されている | - |
| 類似名称 | typosquatting の可能性がないか | Yes |
| 正規性 | 公式リポジトリからの配布か | Yes |

## 参考

- [OpenSSF Concise Guide for Evaluating Open Source Software](https://best.openssf.org/Concise-Guide-for-Evaluating-Open-Source-Software.html)
- [RustSec Advisory Database](https://rustsec.org/)
- [Google Open Source - Rust Crate Audits](https://opensource.googleblog.com/2023/05/open-sourcing-our-rust-crate-audits.html)
