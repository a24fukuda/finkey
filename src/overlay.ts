// Tauri API
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
}

interface Window {
	__TAURI__?: TauriGlobal;
}

const invoke =
	window.__TAURI__?.tauri?.invoke ??
	window.__TAURI__?.invoke ??
	(async <T>(): Promise<T> => {
		throw new Error("Tauri not available");
	});

const listen =
	window.__TAURI__?.event?.listen ??
	(async <T>(
		_event: string,
		_handler: (event: { payload: T }) => void,
	): Promise<() => void> => {
		return () => {};
	});

// DOM要素
const shortcutKeyEl = document.getElementById("shortcut-key") as HTMLElement;
const countdownEl = document.getElementById("countdown") as HTMLElement;

// 状態
let countdownTimer: number | null = null;
let remainingSeconds = 0;

// オーバーレイペイロード
interface OverlayPayload {
	shortcut_key: string;
	duration: number;
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

// 初期化
async function init(): Promise<void> {
	// Tauriイベントリスナー
	try {
		await listen<OverlayPayload>("overlay-show", (event) => {
			const { shortcut_key, duration } = event.payload;

			// ショートカットキーを表示
			shortcutKeyEl.textContent = shortcut_key;

			// カウントダウン開始
			startCountdown(duration);
		});
	} catch (_e) {
		console.log("Failed to register event listener");
	}
}

document.addEventListener("DOMContentLoaded", init);
