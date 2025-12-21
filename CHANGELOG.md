# Changelog

Finkeyの全ての変更履歴を記録します。

フォーマットは[Keep a Changelog](https://keepachangelog.com/ja/1.0.0/)に基づき、
バージョニングは[Semantic Versioning](https://semver.org/lang/ja/)に準拠しています。

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

[0.4.0]: https://github.com/a24fukuda/finkey/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/a24fukuda/finkey/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/a24fukuda/finkey/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/a24fukuda/finkey/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/a24fukuda/finkey/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/a24fukuda/finkey/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/a24fukuda/finkey/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/a24fukuda/finkey/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/a24fukuda/finkey/releases/tag/v0.1.0
