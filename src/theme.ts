import { invoke } from "./tauri-api";

// テーマ設定の型
export type ThemeSetting = "system" | "light" | "dark";

// 現在のテーマ設定
let currentThemeSetting: ThemeSetting = "system";

// システムテーマを取得
export function getSystemTheme(): "light" | "dark" {
	return window.matchMedia("(prefers-color-scheme: dark)").matches
		? "dark"
		: "light";
}

// 現在のテーマ設定を取得
export function getCurrentThemeSetting(): ThemeSetting {
	return currentThemeSetting;
}

// テーマを適用
export function applyTheme(setting?: ThemeSetting): void {
	if (setting !== undefined) {
		currentThemeSetting = setting;
	}

	let effectiveTheme: "light" | "dark";

	if (currentThemeSetting === "system") {
		effectiveTheme = getSystemTheme();
	} else {
		effectiveTheme = currentThemeSetting;
	}

	// data-theme属性を設定（lightの場合のみ属性を追加、darkはデフォルト）
	if (effectiveTheme === "light") {
		document.documentElement.setAttribute("data-theme", "light");
	} else {
		document.documentElement.removeAttribute("data-theme");
	}

	// data-theme-setting属性を設定（CSS切り替え用）
	document.documentElement.setAttribute("data-theme-setting", currentThemeSetting);
}

// バックエンドからテーマ設定を読み込んで適用
export async function loadAndApplyTheme(): Promise<void> {
	try {
		const theme = await invoke<string>("get_theme_setting");
		currentThemeSetting = theme as ThemeSetting;
	} catch (_e) {
		currentThemeSetting = "system";
	}
	applyTheme();
}

// テーマを切り替え（system -> light -> dark -> system）
export async function toggleTheme(): Promise<void> {
	const order: ThemeSetting[] = ["system", "light", "dark"];
	const currentIndex = order.indexOf(currentThemeSetting);
	currentThemeSetting = order[(currentIndex + 1) % order.length];

	applyTheme();

	// 設定を非同期で保存
	try {
		await invoke("set_theme_setting", { theme: currentThemeSetting });
	} catch (_e) {
		// 保存失敗は無視
	}
}

// システムテーマ変更を監視
export function setupSystemThemeListener(): void {
	window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
		if (currentThemeSetting === "system") {
			applyTheme();
		}
	});
}

// ウィンドウフォーカス時にテーマを再読み込み
export function setupWindowFocusListener(): void {
	window.addEventListener("focus", async () => {
		await loadAndApplyTheme();
	});
}

// テーマボタンのタイトルを取得
export function getThemeButtonTitle(): string {
	const titles: Record<ThemeSetting, string> = {
		system: "テーマ: システム設定に従う",
		light: "テーマ: ライト",
		dark: "テーマ: ダーク",
	};
	return titles[currentThemeSetting];
}

// 外部から指定されたテーマを適用（キーガイド画面用）
export function applyThemeFromSetting(setting: string): void {
	const themeSetting = setting as ThemeSetting;
	let effectiveTheme: "light" | "dark";

	if (themeSetting === "system") {
		effectiveTheme = getSystemTheme();
	} else if (themeSetting === "light") {
		effectiveTheme = "light";
	} else {
		effectiveTheme = "dark";
	}

	// data-theme属性を設定（lightの場合のみ属性を追加、darkはデフォルト）
	if (effectiveTheme === "light") {
		document.documentElement.setAttribute("data-theme", "light");
	} else {
		document.documentElement.removeAttribute("data-theme");
	}

	// data-theme-setting属性を設定（CSS切り替え用）
	document.documentElement.setAttribute("data-theme-setting", themeSetting);
}
