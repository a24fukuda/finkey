import { invoke, listen } from "./tauri-api";
import type {
	ActiveWindowInfo,
	NormalizedApp,
	Platform,
	Shortcut,
} from "./types";

// DevToolsショートカットを無効化
document.addEventListener(
	"keydown",
	(e) => {
		// Ctrl+Shift+J, Ctrl+Shift+I, F12, Ctrl+U を無効化
		if (
			(e.ctrlKey &&
				e.shiftKey &&
				(e.key === "J" || e.key === "j" || e.key === "I" || e.key === "i")) ||
			(e.ctrlKey && (e.key === "U" || e.key === "u")) ||
			e.key === "F12"
		) {
			e.preventDefault();
			e.stopPropagation();
			return false;
		}
	},
	true,
);

// 右クリックメニューを無効化
document.addEventListener("contextmenu", (e) => {
	e.preventDefault();
});

// DOM要素
const activeAppNameEl = document.getElementById(
	"active-app-name",
) as HTMLElement;
const searchInput = document.getElementById("search-input") as HTMLInputElement;
const resultsList = document.getElementById("results-list") as HTMLElement;
const noResults = document.getElementById("no-results") as HTMLElement;
const openConfigBtn = document.getElementById(
	"open-config-btn",
) as HTMLButtonElement;
const openSettingsBtn = document.getElementById(
	"open-settings-btn",
) as HTMLButtonElement;
const themeToggleBtn = document.getElementById(
	"theme-toggle",
) as HTMLButtonElement;

// テーマ設定
type ThemeSetting = "system" | "light" | "dark";
let currentThemeSetting: ThemeSetting = "system";

// 状態
let currentPlatform: Platform = "mac";
let selectedIndex = 0;
let filteredShortcuts: Shortcut[] = [];
let activeWindowInfo: ActiveWindowInfo | null = null;
let matchedApps: NormalizedApp[] = [];
let shortcuts: Shortcut[] = [];

// システムテーマを取得（同期的にCSSメディアクエリを使用）
function getSystemTheme(): "light" | "dark" {
	return window.matchMedia("(prefers-color-scheme: dark)").matches
		? "dark"
		: "light";
}

// テーマを適用する
function applyTheme(): void {
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

	// data-theme-setting属性を設定（アイコン切り替え用）
	document.documentElement.setAttribute(
		"data-theme-setting",
		currentThemeSetting,
	);

	// ボタンのtitleを更新
	const titles: Record<ThemeSetting, string> = {
		system: "テーマ: システム設定に従う",
		light: "テーマ: ライト",
		dark: "テーマ: ダーク",
	};
	themeToggleBtn.title = titles[currentThemeSetting];
}

// テーマ設定を切り替え（system -> light -> dark -> system）
function toggleTheme(): void {
	const order: ThemeSetting[] = ["system", "light", "dark"];
	const currentIndex = order.indexOf(currentThemeSetting);
	currentThemeSetting = order[(currentIndex + 1) % order.length];

	// テーマを即座に適用
	applyTheme();

	// 設定を非同期で保存（UIには影響しない）
	invoke("set_theme_setting", { theme: currentThemeSetting }).catch(() => {
		console.log("Failed to save theme setting");
	});
}

// テーマ設定を読み込み
async function loadThemeSetting(): Promise<void> {
	try {
		const theme = await invoke<string>("get_theme_setting");
		currentThemeSetting = theme as ThemeSetting;
	} catch (_e) {
		currentThemeSetting = "system";
	}
	applyTheme();
}

// 初期化
async function init(): Promise<void> {
	// テーマを初期化
	await loadThemeSetting();

	// プラットフォーム検出
	try {
		const platform = await invoke<string>("get_platform");
		currentPlatform = platform === "darwin" ? "mac" : "windows";
	} catch (_e) {
		console.log("Platform detection failed, defaulting to mac");
	}

	// ショートカットデータをバックエンドから読み込む
	try {
		shortcuts = await invoke<Shortcut[]>("get_shortcuts");
	} catch (_e) {
		console.log("Failed to load shortcuts from backend, using empty list");
		shortcuts = [];
	}

	// 初期表示
	filterAndDisplay();

	// イベントリスナー
	searchInput.addEventListener("input", handleTextSearch);
	searchInput.addEventListener("keydown", handleKeydown);
	openConfigBtn.addEventListener("click", openConfigFile);
	openSettingsBtn.addEventListener("click", openSettingsFile);
	themeToggleBtn.addEventListener("click", toggleTheme);

	// Tauriイベントリスナー（アクティブウィンドウ情報を受け取る）
	try {
		await listen<ActiveWindowInfo | null>("window-shown", async (event) => {
			activeWindowInfo = event.payload ?? null;

			// ウィンドウ表示時にテーマを再適用（システム設定が変わっている可能性があるため）
			if (currentThemeSetting === "system") {
				applyTheme();
			}

			// ショートカットデータを再読み込み（設定ファイルが変更されている可能性があるため）
			try {
				shortcuts = await invoke<Shortcut[]>("get_shortcuts");
			} catch (_e) {
				console.log("Failed to reload shortcuts");
			}

			// バックエンドでアプリをマッチング
			try {
				matchedApps = await invoke<NormalizedApp[]>("get_matched_apps", {
					info: activeWindowInfo,
				});
			} catch (_e) {
				console.log("Failed to get matched apps");
				matchedApps = [];
			}

			// UIにアプリ名を表示
			let displayText = "-";
			if (matchedApps.length > 0) {
				displayText = matchedApps.map((app) => app.name).join(", ");
			} else if (activeWindowInfo) {
				displayText = activeWindowInfo.process ?? "-";
			}
			activeAppNameEl.textContent = displayText;

			// 状態をリセット
			selectedIndex = 0;
			searchInput.value = "";
			searchInput.focus();
			searchInput.select();

			filterAndDisplay();
		});
	} catch (_e) {
		// イベントリスナー登録に失敗
	}
}

// テキスト検索処理
function handleTextSearch(): void {
	selectedIndex = 0;
	filterAndDisplay();
}

// キーボードナビゲーション
async function handleKeydown(e: KeyboardEvent): Promise<void> {
	switch (e.key) {
		case "ArrowDown":
			e.preventDefault();
			if (selectedIndex < filteredShortcuts.length - 1) {
				selectedIndex++;
				updateSelection();
				scrollToSelected();
			}
			break;
		case "ArrowUp":
			e.preventDefault();
			if (selectedIndex > 0) {
				selectedIndex--;
				updateSelection();
				scrollToSelected();
			}
			break;
		case "Enter":
			e.preventDefault();
			selectShortcut();
			break;
		case "Escape":
			e.preventDefault();
			hideWindow();
			break;
	}
}

// 選択したショートカットをオーバーレイ表示
async function selectShortcut(): Promise<void> {
	if (filteredShortcuts.length === 0 || selectedIndex < 0) {
		return;
	}

	const shortcut = filteredShortcuts[selectedIndex];
	try {
		await invoke("show_overlay", { shortcutKey: shortcut.key });
	} catch (e) {
		console.log("Failed to show overlay:", e);
	}
}

// ウィンドウを隠す
async function hideWindow(): Promise<void> {
	try {
		await invoke("hide_main_window");
	} catch (_e) {
		console.log("Hide window failed");
	}
}

// ショートカット設定ファイルを開く
async function openConfigFile(): Promise<void> {
	try {
		await invoke("open_config_file");
	} catch (e) {
		console.log("Failed to open config file:", e);
	}
}

// アプリ設定ファイルを開く
async function openSettingsFile(): Promise<void> {
	try {
		await invoke("open_settings_file");
	} catch (e) {
		console.log("Failed to open settings file:", e);
	}
}

// フィルタリングと表示
function filterAndDisplay(): void {
	filterByText();
	displayResults();
}

// OS名を取得
function getOsName(): string {
	return currentPlatform === "mac" ? "macOS" : "Windows";
}

// テキストでフィルタリングとソート
function filterByText(): void {
	const query = searchInput.value.toLowerCase().trim();

	// 検出アプリ名のリストを取得（name で比較）
	const detectedAppNames = matchedApps.map((app) => app.name.toLowerCase());
	const osName = getOsName().toLowerCase();

	// まずタグでフィルタリング（クエリがある場合）
	let filtered = shortcuts;
	if (query) {
		filtered = shortcuts.filter((shortcut) =>
			shortcut.tags.some((tag) => tag.toLowerCase().includes(query)),
		);
	}

	// 優先度でソート: 1. 検出アプリ, 2. OS, 3. その他
	filteredShortcuts = filtered.sort((a, b) => {
		const aApp = a.app.toLowerCase();
		const bApp = b.app.toLowerCase();

		const aIsDetected = detectedAppNames.includes(aApp);
		const bIsDetected = detectedAppNames.includes(bApp);
		const aIsOs = aApp === osName;
		const bIsOs = bApp === osName;

		// 検出アプリを最優先
		if (aIsDetected && !bIsDetected) return -1;
		if (!aIsDetected && bIsDetected) return 1;

		// 次にOS
		if (aIsOs && !bIsOs) return -1;
		if (!aIsOs && bIsOs) return 1;

		// 同じカテゴリ内ではアプリ名でソート
		return aApp.localeCompare(bApp);
	});
}

// 結果表示
function displayResults(): void {
	resultsList.innerHTML = "";

	if (filteredShortcuts.length === 0) {
		noResults.style.display = "block";
		return;
	}

	noResults.style.display = "none";

	const fragment = document.createDocumentFragment();

	filteredShortcuts.forEach((shortcut, index) => {
		const item = createResultItem(shortcut, index);
		fragment.appendChild(item);
	});

	resultsList.appendChild(fragment);
	updateSelection();
}

// 結果アイテム作成
function createResultItem(shortcut: Shortcut, index: number): HTMLDivElement {
	const item = document.createElement("div");
	item.className = "result-item";
	if (index === selectedIndex) item.classList.add("selected");
	item.dataset.index = String(index);

	const icon = shortcut.icon;
	const displayKey = shortcut.key;
	const appLabel = shortcut.app;

	// ハイライト処理
	const query = searchInput.value.toLowerCase().trim();
	const highlightedAction = highlightText(shortcut.action, query);

	item.innerHTML = `
    <div class="result-icon">${escapeHtml(icon)}</div>
    <div class="result-content">
      <div class="result-action">${highlightedAction}</div>
      <span class="result-category">${escapeHtml(appLabel)}</span>
    </div>
    <div class="result-shortcut">
      <span class="shortcut-key ${currentPlatform}">${escapeHtml(displayKey)}</span>
    </div>
  `;

	item.addEventListener("click", () => {
		selectedIndex = index;
		updateSelection();
		selectShortcut();
	});

	return item;
}

// テキストハイライト
function highlightText(text: string, query: string): string {
	if (!query) return escapeHtml(text);

	const escaped = escapeHtml(text);
	const escapedQuery = escapeHtml(query);
	const regex = new RegExp(`(${escapeRegExp(escapedQuery)})`, "gi");
	return escaped.replace(regex, '<span class="highlight">$1</span>');
}

// HTMLエスケープ
function escapeHtml(text: string): string {
	const div = document.createElement("div");
	div.textContent = text;
	return div.innerHTML;
}

// 正規表現エスケープ
function escapeRegExp(string: string): string {
	return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// 選択状態更新
function updateSelection(): void {
	document.querySelectorAll(".result-item").forEach((item, index) => {
		item.classList.toggle("selected", index === selectedIndex);
	});
}

// 選択アイテムにスクロール
function scrollToSelected(): void {
	const selected = document.querySelector(".result-item.selected");
	if (selected) {
		selected.scrollIntoView({ block: "nearest", behavior: "auto" });
	}
}

// 初期化実行
document.addEventListener("DOMContentLoaded", init);
