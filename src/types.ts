// Tauri バックエンドの型定義

import { type OsTypeValue } from "./constants";

/** 正規化されたショートカット（フロントエンドで使用） */
export interface Shortcut {
	app: string;
	icon: string;
	action: string;
	key: string;
	tags: string[];
}

/** OS種別 */
export type OsType = OsTypeValue;

/** キーバインド設定 */
export interface Keybinding {
	action: string;
	key: string;
	tags?: string[];
}

/** アプリ設定（生データ） */
export interface AppConfig {
	icon?: string;
	name?: string;
	bind?: string | string[];
	os?: OsType;
	keybindings: Keybinding[];
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
	| "open_config_file"
	| "open_settings_file"
	| "open_keybindings_window"
	| "close_keybindings_window"
	| "get_theme_setting"
	| "set_theme_setting"
	| "get_system_theme"
	| "show_overlay"
	| "hide_overlay"
	| "get_keybindings_raw"
	| "save_keybindings"
	| "reset_keybindings";

/** Tauri イベント名 */
export type TauriEvent = "window-shown" | "window-hidden" | "overlay-show";
