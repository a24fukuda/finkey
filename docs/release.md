# リリース

GitHub Actionsを使用したリリースワークフローの操作手順と設定を定める。

## 概要

本プロジェクトでは署名付きタグとTauri自動アップデートを使用してリリースを行う。

### 操作フロー

```
/bump-version でバージョン更新
         │
         ▼
    PRをmainにマージ
         │
         ▼
   CI自動実行 ──────► 失敗時は修正してやり直し
         │
         ▼ 成功
./scripts/release-tag.ps1 実行
         │
         ▼
 Release自動実行 ───► リリース公開
         │
         ▼
   ユーザーに自動更新通知
```

## リリース手順

### 1. バージョンを更新

`/bump-version` コマンドでバージョンを更新し、mainブランチにマージする。

### 2. CIでビルド成功を確認

mainブランチへのマージ時にCIワークフローが自動実行される。ビルドが成功することを確認する。

### 3. 署名付きタグを作成してプッシュ

リリースタグスクリプトを実行する:

```powershell
./scripts/release-tag.ps1
```

スクリプトは `package.json` からバージョンを取得し、署名付きタグを作成してプッシュする。タグのプッシュにより自動的にReleaseワークフローが実行され、リリースが公開される。

## 初期設定

以下は一度だけ行う設定である。

### シークレットの登録

リポジトリの **Settings** → **Secrets and variables** → **Actions** に以下を登録する。

| シークレット名 | 用途 |
|---------------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | Tauriアップデーター署名用秘密鍵 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 上記秘密鍵のパスワード |

### 署名キーの設定

署名付きタグを作成するには、SSHキーをGitHubアカウントに署名キーとして登録する必要がある。

1. SSHキーをGitHubに署名キーとして登録:

**Settings** → **SSH and GPG keys** → **New SSH key** → Key type: **Signing Key**

2. Gitで署名を設定:

```bash
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
```

### Tauri自動アップデートの設定

#### 署名鍵の生成

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

#### 公開鍵の設定

生成された公開鍵を `src-tauri/tauri.conf.json` の `plugins.updater.pubkey` に設定する。

```bash
cat ~/.tauri/finkey.key.pub
```

#### シークレットへの登録

1. 秘密鍵の内容を確認:

```bash
cat ~/.tauri/finkey.key
```

2. 以下のシークレットを登録:

| シークレット名 | 値 |
|---------------|-----|
| `TAURI_SIGNING_PRIVATE_KEY` | 秘密鍵の内容全体 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 鍵生成時に設定したパスワード |

## ワークフロー詳細

本プロジェクトでは2つのワークフローを使用する。

| ワークフロー | トリガー | 目的 |
|-------------|---------|------|
| CI | mainブランチへのプッシュ/PR | ビルド検証 |
| Release | `v*` タグのプッシュ | リリースビルド |

### CI

mainブランチへのプッシュまたはPR作成時に自動実行される。TypeScriptビルドとTauriアプリのビルドを検証する。

### Release

`v*` 形式のタグがプッシュされると自動実行される。以下を行う:

- Tauriアプリのリリースビルド
- アップデーター署名の付与
- リリースの作成・公開
- アセット（.msi, .exe）のアップロード

## 動作確認

### CIワークフロー

1. mainブランチにプッシュまたはPRを作成
2. CIワークフローが自動実行されることを確認
3. ビルドが成功することを確認

### 署名付きタグ

1. 署名付きタグを作成してプッシュ
2. GitHubでタグに「Verified」バッジが表示されていることを確認

### Releaseワークフロー

1. `v*` 形式のタグをプッシュ
2. Releaseワークフローが自動実行されることを確認
3. リリースが公開されていることを確認
4. アセット（.msi, .exe）がアップロードされていることを確認

### 自動アップデート

1. 古いバージョンのアプリを起動
2. アップデート通知が表示されることを確認

## 参考

- [Signing tags - GitHub Docs](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-tags)
- [Tauri Updater - Signing updates](https://v2.tauri.app/plugin/updater/#signing-updates)
