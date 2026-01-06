# GitHub Actions リリース設定

GitHub Actionsでリリースワークフローを実行するための設定手順を定める。

## 概要

リリースワークフローでは以下の機能を使用する:

| 機能 | 説明 |
|------|------|
| 署名付きタグ | GitHubでのVerifiedバッジ表示 |
| Tauri自動アップデート | アプリの自動更新機能 |

## 必要なシークレット

リポジトリの **Settings** → **Secrets and variables** → **Actions** に以下を登録する。

| シークレット名 | 用途 | 必須 |
|---------------|------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | Tauriアップデーター署名用秘密鍵 | ○ |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 上記秘密鍵のパスワード | ○ |

## 署名付きタグについて

リリースワークフローはGitHub Release API経由でドラフトリリースを作成する。Release APIでリリースを作成すると、指定したコミットにタグが自動作成され、GitHub自体が署名を付与するため、追加の設定なしで自動的にVerifiedバッジが表示される。

**注意:** Git Tags API（`/repos/{owner}/{repo}/git/tags`）ではタグに署名が付与されない。Verifiedバッジを表示するにはRelease API（`/repos/{owner}/{repo}/releases`）を使用する必要がある。

## Tauri自動アップデートの設定

### 署名鍵の生成

Tauri CLIで署名鍵ペアを生成する:

```bash
npx tauri signer generate -w ~/.tauri/finkey.key
```

実行するとパスワードの入力を求められる。このパスワードは `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` として登録する。

生成されるファイル:

| ファイル | 内容 |
|---------|------|
| `~/.tauri/finkey.key` | 秘密鍵 |
| `~/.tauri/finkey.key.pub` | 公開鍵 |

### 公開鍵の設定

生成された公開鍵を `src-tauri/tauri.conf.json` の `plugins.updater.pubkey` に設定する。

```bash
cat ~/.tauri/finkey.key.pub
```

### シークレットの登録

1. 秘密鍵の内容を確認:

```bash
cat ~/.tauri/finkey.key
```

2. 以下のシークレットを登録:

| シークレット名 | 値 |
|---------------|-----|
| `TAURI_SIGNING_PRIVATE_KEY` | 秘密鍵の内容全体 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 鍵生成時に設定したパスワード |

## 動作確認

### 署名付きタグ

1. リリースワークフローを実行
2. 作成されたタグをGitHubで確認
3. タグに「Verified」バッジが表示されていることを確認

### 自動アップデート

1. リリースを公開
2. 古いバージョンのアプリを起動
3. アップデート通知が表示されることを確認

## 参考

- [Releases - GitHub REST API](https://docs.github.com/en/rest/releases/releases)
- [Tauri Updater - Signing updates](https://v2.tauri.app/plugin/updater/#signing-updates)
