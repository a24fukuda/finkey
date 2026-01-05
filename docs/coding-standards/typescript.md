# TypeScriptコーディング規約

本プロジェクトにおけるTypeScriptコードの記述ルール。

## 準拠元

[Biomeデフォルト設定](https://biomejs.dev/reference/configuration/)に準拠する。

## 命名規則

| 対象 | 規則 | 例 |
|------|------|-----|
| 関数・メソッド | camelCase | `handleKeydown`, `filterByText` |
| 変数・定数 | camelCase | `selectedIndex`, `filteredShortcuts` |
| 型・インターフェース | PascalCase | `ActiveWindowInfo`, `Shortcut` |
| 定数（モジュールレベル） | SCREAMING_SNAKE_CASE | `MACOS_NAME`, `WINDOWS_NAME` |
| DOM要素 | camelCase + El接尾辞（推奨） | `searchInputEl`, `resultsListEl` |
| イベントハンドラ | handle + 動詞 | `handleToggleTheme`, `handleKeydown` |

> **Note**: 「推奨」はBiomeで強制されない。既存コードとの一貫性を優先する。

## コード構成

### ファイル構造

```
src/
  types.ts        # 型定義
  constants.ts    # 定数定義
  tauri-api.ts    # Tauri APIラッパー
  theme.ts        # テーマ機能
  search.ts       # 検索画面
  *.css           # 各画面のスタイル
  *.html          # 各画面のHTML
```

### インポート順序

Biome の `organizeImports` によりソースパスのアルファベット順に自動整列される。

```typescript
import { MACOS_NAME, WINDOWS_NAME } from "./constants";
import { invoke, listen } from "./tauri-api";
import { applyTheme, toggleTheme } from "./theme";
import type { ActiveWindowInfo, Shortcut } from "./types";
import { checkAndInstallUpdate } from "./updater";
```

型インポートは `type` キーワードを付けて明示する。

## 型定義

### DOM要素の型アサーション

DOM要素取得時は適切な型でアサーションする。

```typescript
const searchInputEl = document.getElementById("search-input") as HTMLInputElement;
const resultsListEl = document.getElementById("results-list") as HTMLElement;
```

### 関数の戻り値型

明示的に戻り値型を指定する。

```typescript
async function init(): Promise<void> {
    // ...
}

function getOsName(): string {
    return currentPlatform === "mac" ? MACOS_NAME : WINDOWS_NAME;
}
```

### 型定義ファイル

共有する型は `types.ts` に集約する。

## エラーハンドリング

外部API呼び出しは `try-catch` で囲む。

```typescript
try {
    shortcuts = await invoke<Shortcut[]>("get_shortcuts");
} catch (_e) {
    console.log("Failed to load shortcuts");
    shortcuts = [];
}
```

## 非同期処理

Promiseは `async/await` で処理する。コールバックチェーンは使用しない。

```typescript
async function handleToggleTheme(): Promise<void> {
    await toggleTheme();
    themeToggleBtnEl.title = getThemeButtonTitle();
}
```

独立した非同期処理は `Promise.all` で並行実行する。

```typescript
const [shortcuts, settings] = await Promise.all([
    invoke<Shortcut[]>("get_shortcuts"),
    invoke<AppSettings>("get_settings"),
]);
```

## イベントハンドリング

`DOMContentLoaded` 後に初期化関数を実行する。

```typescript
document.addEventListener("DOMContentLoaded", init);
```

イベントハンドラはイベントオブジェクトを受け取り、必要に応じて `preventDefault` を呼ぶ。

```typescript
function handleKeydown(e: KeyboardEvent): void {
    switch (e.key) {
        case "ArrowDown":
            e.preventDefault();
            // ...
            break;
    }
}
```

## セキュリティ

ユーザー入力をDOMに挿入する際は必ずエスケープする。

```typescript
function escapeHtml(text: string): string {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
}
```

## 品質管理

コミット前に以下を確認:

```
[ ] npm run format
[ ] npm run check（typecheck + lint）
[ ] npm run build:ts
```

## 参考

- [Biome Configuration](https://biomejs.dev/reference/configuration/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/)
