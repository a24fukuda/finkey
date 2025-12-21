import { invoke, openUrl } from "./tauri-api";
import { loadAndApplyTheme, setupSystemThemeListener, setupWindowFocusListener } from "./theme";

// バージョン情報を表示
async function showVersion(): Promise<void> {
	const version = await invoke<string>("get_app_version");
	const versionEl = document.getElementById("app-version");
	if (versionEl) {
		versionEl.textContent = `Version ${version}`;
	}
}

// 初期化
document.addEventListener("DOMContentLoaded", async () => {
	// テーマを適用
	await loadAndApplyTheme();
	setupSystemThemeListener();
	setupWindowFocusListener();

	// バージョン情報を表示
	await showVersion();

	// 閉じるボタン
	document.getElementById("close-btn")?.addEventListener("click", () => {
		invoke("close_about_window");
	});

	// GitHubリンク
	document.getElementById("github-link")?.addEventListener("click", (e) => {
		e.preventDefault();
		openUrl("https://github.com/a24fukuda/finkey");
	});

	// ESCキーで閉じる
	document.addEventListener("keydown", (e) => {
		if (e.key === "Escape") {
			invoke("close_about_window");
		}
	});
});
