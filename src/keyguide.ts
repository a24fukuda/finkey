import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke, listen } from "./tauri-api";
import { applyThemeFromSetting } from "./theme";

// DOM要素
const appNameEl = document.getElementById("app-name") as HTMLElement;
const actionNameEl = document.getElementById("action-name") as HTMLElement;
const shortcutKeyEl = document.getElementById("shortcut-key") as HTMLElement;
const countdownEl = document.getElementById("countdown") as HTMLElement;
const overlayEl = document.getElementById("overlay") as HTMLElement;

// 状態
let countdownTimer: number | null = null;
let remainingSeconds = 0;

// オーバーレイペイロード
interface OverlayPayload {
	app_name: string;
	action_name: string;
	shortcut_key: string;
	duration: number;
	theme: string;
}

// HTMLエスケープ
function escapeHtml(text: string): string {
	const div = document.createElement("div");
	div.textContent = text;
	return div.innerHTML;
}

// ショートカットキーをパースしてHTML要素を生成
function formatShortcutKey(key: string): string {
	// 順次入力キー（→）で分割
	const sequences = key.split(" → ");

	const formattedSequences = sequences.map((seq) => {
		// 同時押しキー（+）で分割
		const keys = seq.split(" + ");

		const formattedKeys = keys.map(
			(k) => `<kbd class="key-box">${escapeHtml(k.trim())}</kbd>`,
		);

		return formattedKeys.join('<span class="key-separator">+</span>');
	});

	return formattedSequences.join(
		'<span class="key-separator sequence">→</span>',
	);
}

// カウントダウン更新
function updateCountdown(): void {
	if (remainingSeconds > 0) {
		countdownEl.textContent = `${remainingSeconds}秒後に閉じます`;
	} else {
		countdownEl.textContent = "";
	}
}

// カウントダウン開始
function startCountdown(duration: number): void {
	// 既存のタイマーをクリア
	if (countdownTimer !== null) {
		clearInterval(countdownTimer);
	}

	remainingSeconds = duration;
	updateCountdown();

	countdownTimer = window.setInterval(() => {
		remainingSeconds--;
		updateCountdown();

		if (remainingSeconds <= 0) {
			closeOverlay();
		}
	}, 1000);
}

// オーバーレイを閉じる
async function closeOverlay(): Promise<void> {
	// タイマーをクリア
	if (countdownTimer !== null) {
		clearInterval(countdownTimer);
		countdownTimer = null;
	}

	try {
		await invoke("hide_overlay");
	} catch (_e) {
		console.log("Failed to hide overlay");
	}
}

// ウィンドウ位置の保存（デバウンス用）
let savePositionTimer: number | null = null;

// ウィンドウのドラッグを開始
async function startDragging(): Promise<void> {
	try {
		await getCurrentWindow().startDragging();
	} catch (_e) {
		console.log("Failed to start dragging");
	}
}

// ウィンドウ位置を保存
async function savePosition(): Promise<void> {
	try {
		const position = await getCurrentWindow().outerPosition();
		if (position) {
			await invoke("save_overlay_position", { x: position.x, y: position.y });
		}
	} catch (_e) {
		console.log("Failed to save overlay position");
	}
}

// デバウンス付きで位置を保存
function savePositionDebounced(): void {
	if (savePositionTimer !== null) {
		clearTimeout(savePositionTimer);
	}
	savePositionTimer = window.setTimeout(() => {
		savePosition();
		savePositionTimer = null;
	}, 300);
}

// 初期化
async function init(): Promise<void> {
	// マウスダウンでドラッグ開始
	overlayEl.addEventListener("mousedown", (e) => {
		// 左クリックのみ
		if (e.button === 0) {
			startDragging();
		}
	});

	// Tauriイベントリスナー
	try {
		await listen<OverlayPayload>("overlay-show", (event) => {
			const { app_name, action_name, shortcut_key, duration, theme } =
				event.payload;

			// テーマを適用
			applyThemeFromSetting(theme);

			// アプリ名とアクション名を表示
			appNameEl.textContent = app_name;
			actionNameEl.textContent = action_name;

			// ショートカットキーを表示（個別ボックス形式）
			shortcutKeyEl.innerHTML = formatShortcutKey(shortcut_key);

			// カウントダウン開始
			startCountdown(duration);
		});

		// ウィンドウ移動イベントをリッスン
		await listen("tauri://move", () => {
			savePositionDebounced();
		});
	} catch (_e) {
		console.log("Failed to register event listener");
	}
}

document.addEventListener("DOMContentLoaded", init);
