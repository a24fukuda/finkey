// Tauri API 共通モジュール

/** Tauri グローバルオブジェクトの型定義 */
interface TauriGlobal {
	tauri?: {
		invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
	};
	invoke?: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
	event?: {
		listen: <T>(
			event: string,
			handler: (event: { payload: T }) => void,
		) => Promise<() => void>;
	};
	shell?: {
		open: (url: string) => Promise<void>;
	};
}

declare global {
	interface Window {
		__TAURI__?: TauriGlobal;
	}
}

/**
 * Tauri コマンドを呼び出す
 * Tauri環境外ではエラーをスロー
 */
export const invoke =
	window.__TAURI__?.tauri?.invoke ??
	window.__TAURI__?.invoke ??
	(async <T>(): Promise<T> => {
		throw new Error("Tauri not available");
	});

/**
 * Tauri イベントをリッスンする
 * Tauri環境外では何もしない関数を返す
 */
export const listen =
	window.__TAURI__?.event?.listen ??
	(async <T>(
		_event: string,
		_handler: (event: { payload: T }) => void,
	): Promise<() => void> => {
		return () => {};
	});

/**
 * URLをデフォルトブラウザで開く
 * Tauri環境外では何もしない
 */
export async function openUrl(url: string): Promise<void> {
	await window.__TAURI__?.shell?.open(url);
}
