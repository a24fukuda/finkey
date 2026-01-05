# CSSコーディング規約

本プロジェクトにおけるCSSコードの記述ルール。

## 準拠元

[Biomeデフォルト設定](https://biomejs.dev/reference/configuration/)に準拠する。

## 命名規則

### クラス名

ケバブケース（小文字、ハイフン区切り）を使用する。

```css
.search-container { }
.result-item { }
.no-results-hint { }
```

### BEM風の命名（推奨）

コンポーネント内の要素はコンポーネント名をプレフィックスにする。

```css
.result-item { }          /* ブロック */
.result-icon { }          /* 要素 */
.result-item.selected { } /* 修飾子 */
```

> **Note**: 本プロジェクトでは厳密なBEMではなく、簡略化した命名を採用している。

## カスタムプロパティ

### 定義場所

`:root` でグローバルに定義し、テーマごとに上書きする。

```css
:root {
	--bg-primary: rgba(28, 28, 30, 0.98);
	--text-primary: #f5f5f7;
}

[data-theme="light"] {
	--bg-primary: rgba(255, 255, 255, 0.98);
	--text-primary: #1c1c1e;
}
```

### 命名規則

| カテゴリ | プレフィックス | 例 |
|----------|----------------|-----|
| 背景色 | `--bg-` | `--bg-primary`, `--bg-hover` |
| 文字色 | `--text-` | `--text-primary`, `--text-muted` |
| ボーダー | `--border-` | `--border-color` |
| アクセント | `--accent-` | `--accent-color` |
| 角丸 | `--radius` | `--radius`, `--radius-sm` |
| 影 | `--shadow-` | `--shadow-color` |
| ショートカット | `--shortcut-` | `--shortcut-key-color` |

## テーマ対応

HTMLの `data-theme` 属性でテーマを切り替える。

```css
:root {
	--bg-primary: rgba(28, 28, 30, 0.98);  /* ダーク */
}

[data-theme="light"] {
	--bg-primary: rgba(255, 255, 255, 0.98);  /* ライト */
}
```

## プラットフォーム固有スタイル

クラスでプラットフォームを区別する。

```css
.key-box.mac {
	border-color: rgba(0, 122, 255, 0.3);
	background: rgba(0, 122, 255, 0.1);
}

.key-box.windows {
	border-color: rgba(0, 120, 212, 0.3);
	background: rgba(0, 120, 212, 0.1);
}
```

## プロパティ順序（推奨）

以下の順序で記述する:

1. レイアウト（`display`, `flex`, `grid`）
2. ボックスモデル（`width`, `margin`, `padding`）
3. 位置（`position`, `top`, `z-index`）
4. 装飾（`background`, `border`, `color`）
5. タイポグラフィ（`font-*`, `text-*`）
6. その他（`cursor`, `transition`）

```css
.result-item {
	display: flex;
	align-items: center;
	padding: 10px 14px;
	border-radius: var(--radius-sm);
	cursor: pointer;
	transition: background 0.1s ease;
}
```

> **Note**: 「推奨」はBiomeで強制されない。可読性向上のためのガイドラインとして参照する。

## 品質管理

コミット前に以下を確認:

```
[ ] npm run format
[ ] npm run check
[ ] 各テーマ（ライト/ダーク）で表示確認
[ ] 各プラットフォーム（Mac/Windows）で表示確認
```

## 参考

- [Biome Configuration](https://biomejs.dev/reference/configuration/)
- [MDN CSS Reference](https://developer.mozilla.org/docs/Web/CSS/Reference)
