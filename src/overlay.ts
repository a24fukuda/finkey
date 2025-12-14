import { invoke, listen } from "./tauri-api";

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
	theme: string;
}

// システムテーマを取得
function getSystemTheme(): "light" | "dark" {
	return window.matchMedia("(prefers-color-scheme: dark)").matches
		? "dark"
		: "light";
}

// テーマを適用
function applyTheme(themeSetting: string): void {
	let effectiveTheme: "light" | "dark";

	if (themeSetting === "system") {
		effectiveTheme = getSystemTheme();
	} else if (themeSetting === "light") {
		effectiveTheme = "light";
	} else {
		effectiveTheme = "dark";
	}

	if (effectiveTheme === "light") {
		document.documentElement.setAttribute("data-theme", "light");
	} else {
		document.documentElement.removeAttribute("data-theme");
	}
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
			(k) => `<kbd class="overlay-key-box">${escapeHtml(k.trim())}</kbd>`,
		);

		return formattedKeys.join('<span class="overlay-key-separator">+</span>');
	});

	return formattedSequences.join(
		'<span class="overlay-key-separator sequence">→</span>',
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

// 初期化
async function init(): Promise<void> {
	// Tauriイベントリスナー
	try {
		await listen<OverlayPayload>("overlay-show", (event) => {
			const { shortcut_key, duration, theme } = event.payload;

			// テーマを適用
			applyTheme(theme);

			// ショートカットキーを表示（個別ボックス形式）
			shortcutKeyEl.innerHTML = formatShortcutKey(shortcut_key);

			// カウントダウン開始
			startCountdown(duration);
		});
	} catch (_e) {
		console.log("Failed to register event listener");
	}
}

document.addEventListener("DOMContentLoaded", init);
