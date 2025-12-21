// Tauri API 共通モジュール（Tauri v2対応）

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";
import { open as shellOpen } from "@tauri-apps/plugin-shell";

/**
 * Tauri コマンドを呼び出す
 */
export const invoke = tauriInvoke;

/**
 * Tauri イベントをリッスンする
 */
export const listen = tauriListen;

/**
 * URLをデフォルトブラウザで開く
 */
export async function openUrl(url: string): Promise<void> {
	await shellOpen(url);
}
