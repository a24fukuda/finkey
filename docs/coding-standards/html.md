# HTMLコーディング規約

本プロジェクトにおけるHTMLコードの記述ルール。

## 準拠元

[Biomeデフォルト設定](https://biomejs.dev/reference/configuration/)に準拠する（実験的サポート）。

## 基本構造

```html
<!DOCTYPE html>
<html lang="ja" data-theme-setting="system">
<head>
	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>ページタイトル</title>
	<link rel="stylesheet" href="styles.css">
</head>
<body>
	<div id="app">
		<!-- コンテンツ -->
	</div>
	<script type="module" src="script.js"></script>
</body>
</html>
```

## ID・クラス命名

IDはケバブケースを使用する。JavaScriptから参照する要素に付与する。

```html
<input type="text" id="search-input">
<div id="results-list"></div>
```

## データ属性

### テーマ管理

`data-theme` と `data-theme-setting` でテーマを管理する。

```html
<html lang="ja" data-theme-setting="system">
```

## SVGアイコン

アイコンはインラインSVGで記述する。`currentColor` でCSS継承を利用する。

```html
<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
	<circle cx="11" cy="11" r="8"></circle>
	<path d="m21 21-4.35-4.35"></path>
</svg>
```

## アクセシビリティ

アイコンのみのボタンには `title` 属性でラベルを提供する。

```html
<button class="config-btn" id="open-config-btn" title="ショートカット設定を開く">
	<svg>...</svg>
</button>
```

## Tauri固有

ドラッグ可能領域はCSSで `webkit-app-region: drag` を設定する。入力要素は `no-drag` で除外する。

## 品質管理

コミット前に以下を確認:

```
[ ] npm run format
[ ] npm run check
[ ] 各ブラウザで表示確認
```

## 参考

- [Biome Configuration](https://biomejs.dev/reference/configuration/)
- [MDN HTML Reference](https://developer.mozilla.org/docs/Web/HTML)
