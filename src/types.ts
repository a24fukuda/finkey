// Tauri バックエンドの型定義

/** 正規化されたショートカット（フロントエンドで使用） */
export interface Shortcut {
	app: string;
	icon: string;
	action: string;
	key: string;
	tags: string[];
}

/** アクティブウィンドウ情報 */
export interface ActiveWindowInfo {
	process?: string;
	window?: string;
}

/** 正規化されたアプリ情報（マッチしたアプリ） */
export interface NormalizedApp {
	name: string;
	icon: string;
}

/** プラットフォーム種別 */
export type Platform = "mac" | "windows";

/** Tauri invoke コマンド名 */
export type TauriCommand =
	| "get_platform"
	| "get_shortcuts"
	| "get_matched_apps"
	| "hide_main_window"
	| "open_config_file";

/** Tauri イベント名 */
export type TauriEvent = "window-shown" | "window-hidden";

/** window-shown イベントのペイロード */
export interface WindowShownPayload extends ActiveWindowInfo {}
