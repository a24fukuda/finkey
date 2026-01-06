# Changelog

Finkeyの全ての変更履歴を記録します。

フォーマットは[Keep a Changelog](https://keepachangelog.com/ja/1.0.0/)に基づき、
バージョニングは[Semantic Versioning](https://semver.org/lang/ja/)に準拠しています。

## [0.4.8] - 2026-01-06

### Changed

- リリースワークフローを簡素化
  - タグプッシュによる自動リリースに変更
  - ドラフトリリースを廃止し即時公開に変更
  - CIワークフローを追加（mainブランチのビルド検証）
- リリースタグの署名方式をローカル署名に変更
  - Git Tags APIではVerifiedバッジが表示されないため

### Developer Experience

- release-tag.ps1スクリプトを追加
- スクリプトをscripts/ディレクトリに統合
- リリースドキュメントを操作フロー中心に再構成
- 署名設定をSSHのみに統一

## [0.4.7] - 2026-01-06

### Changed

- リリースタグにSSH署名を追加してVerifiedバッジが表示されるように改善

## [0.4.6] - 2026-01-06

### Fixed

- Tauri v2用の署名設定を修正
  - 環境変数名を正しい形式に変更
  - createUpdaterArtifactsを有効化

## [0.4.5] - 2026-01-06

### Fixed

- 自動アップデート用のlatest.jsonが生成されない問題を修正
  - tauri-actionのオプション名を修正

## [0.4.4] - 2026-01-05

### Fixed

- 自動アップデートが動作しない問題を修正
  - リリースワークフローにupdaterオプションを追加

## [0.4.3] - 2026-01-05

### Developer Experience

- Claude Codeのbump-versionコマンドを追加

## [0.4.2] - 2025-12-23

### Added

- トレイメニューに「アップデートを確認」ボタンを追加 (#22)
  - ユーザーが能動的にアップデートを確認・実行可能に

## [0.4.1] - 2025-12-23

### Changed

- Tauri 2.0への移行
  - パフォーマンスと安定性の向上
  - macOSでの透明ウィンドウの安定性改善
  - 新しいプラグインアーキテクチャへの対応

### Removed

- 自動更新機能を一時的に削除（Tauri 2.0のプラグイン形式で再実装予定）

## [0.4.0] - 2025-12-21

### Added

- 多重起動防止機能を追加 (#16)
- バージョン情報画面を追加 (#15)
- キーバインド設定画面へのアクセス改善 (#14)

## [0.3.0] - 2025-12-19

### Added

- キーバインド設定画面を追加 (#12)

## [0.2.1] - 2025-12-18

### Changed

- リリースワークフローを手動実行方式（workflow_dispatch）に変更 (#9, #10)
- リリースタグにコミットログを自動で含めるように改善

## [0.2.0] - 2025-12-17

### Added

- オーバーレイウィンドウにショートカット情報（アプリ名、アクション名、キー）を表示 (#7)

## [0.1.4] - 2025-12-16

### Added

- オーバーレイウィンドウのドラッグ移動機能
- ウィンドウ位置の保存・復元機能 (#6)

## [0.1.3] - 2025-12-15

### Fixed

- リリースワークフローにcontents write権限を追加 (#5)

## [0.1.2] - 2025-12-15

### Fixed

- Windowsビルドエラーの修正 (#4)

### Changed

- macOSサポートを一時停止 (#4)

## [0.1.1] - 2025-12-15

### Fixed

- Tauriの自動配信用のキー名を更新 (#3)

## [0.1.0] - 2025-12-15

初期リリース

### Added

- Spotlight風のショートカットキー検索ウィンドウ
- アクティブアプリケーションの自動検出（Windows）
- アプリ別ショートカット設定（`shortcuts.json`）
- アプリ設定の外部化（`apps.json`）
- テーマ切り替え機能（ライト/ダーク）
- ホットキーでの起動（`Ctrl+Shift+Space`）
- オーバーレイウィンドウでのショートカット表示
- 設定ファイルを開く機能
- 自動更新機能 (#2)
- リリース自動化 (#1)

### Security

- CSP設定
- XSS対策
- コマンドインジェクション対策

### Documentation

- README
- 開発ドキュメント
- バージョニング・ブランチ戦略ドキュメント

### Developer Experience

- TypeScript導入
- Biome（リンター/フォーマッター）導入

[0.4.8]: https://github.com/a24fukuda/finkey/compare/v0.4.7...v0.4.8
[0.4.7]: https://github.com/a24fukuda/finkey/compare/v0.4.6...v0.4.7
[0.4.6]: https://github.com/a24fukuda/finkey/compare/v0.4.5...v0.4.6
[0.4.5]: https://github.com/a24fukuda/finkey/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/a24fukuda/finkey/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/a24fukuda/finkey/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/a24fukuda/finkey/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/a24fukuda/finkey/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/a24fukuda/finkey/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/a24fukuda/finkey/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/a24fukuda/finkey/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/a24fukuda/finkey/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/a24fukuda/finkey/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/a24fukuda/finkey/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/a24fukuda/finkey/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/a24fukuda/finkey/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/a24fukuda/finkey/releases/tag/v0.1.0
