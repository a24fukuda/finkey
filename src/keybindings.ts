import {
	DEFAULT_APP_ICON,
	getOsIcon,
	getOsName,
	OsType,
	UNNAMED_APP,
} from "./constants";
import { invoke } from "./tauri-api";
import {
	loadAndApplyTheme,
	setupSystemThemeListener,
	setupWindowFocusListener,
} from "./theme";
import type { AppConfig, Keybinding, OsType as OsTypeValue } from "./types";

// DOM要素
const closeBtn = document.getElementById("close-btn") as HTMLButtonElement;
const appSearchInput = document.getElementById(
	"app-search",
) as HTMLInputElement;
const appListEl = document.getElementById("app-list") as HTMLElement;
const platformListEl = document.getElementById("platform-list") as HTMLElement;
const addBtn = document.getElementById("add-btn") as HTMLButtonElement;
const addMenu = document.getElementById("add-menu") as HTMLElement;
const addAppBtn = document.getElementById("add-app-btn") as HTMLButtonElement;
const addPlatformBtn = document.getElementById(
	"add-platform-btn",
) as HTMLButtonElement;
const noSelectionEl = document.getElementById("no-selection") as HTMLElement;
const editAreaEl = document.getElementById("edit-area") as HTMLElement;
const editIconEl = document.getElementById("edit-icon") as HTMLElement;
const editTypeEl = document.getElementById("edit-type") as HTMLElement;
const deleteAppBtn = document.getElementById(
	"delete-app-btn",
) as HTMLButtonElement;
const appFormEl = document.getElementById("app-form") as HTMLElement;
const platformFormEl = document.getElementById("platform-form") as HTMLElement;
const inputIcon = document.getElementById("input-icon") as HTMLInputElement;
const inputName = document.getElementById("input-name") as HTMLInputElement;
const bindTagsEl = document.getElementById("bind-tags") as HTMLElement;
const bindInput = document.getElementById("bind-input") as HTMLInputElement;
const inputOs = document.getElementById("input-os") as HTMLSelectElement;
const inputPlatformIcon = document.getElementById(
	"input-platform-icon",
) as HTMLInputElement;
const keybindingsTbody = document.getElementById(
	"keybindings-tbody",
) as HTMLElement;
const addKeybindingBtn = document.getElementById(
	"add-keybinding-btn",
) as HTMLButtonElement;
const resetBtn = document.getElementById("reset-btn") as HTMLButtonElement;
const openFileBtn = document.getElementById(
	"open-file-btn",
) as HTMLButtonElement;
const cancelBtn = document.getElementById("cancel-btn") as HTMLButtonElement;
const saveBtn = document.getElementById("save-btn") as HTMLButtonElement;

// キーキャプチャモーダル
const keyCaptureModal = document.getElementById(
	"key-capture-modal",
) as HTMLElement;
const capturedKeyEl = document.getElementById("captured-key") as HTMLElement;
const sequenceModeCheckbox = document.getElementById(
	"sequence-mode",
) as HTMLInputElement;
const clearKeyBtn = document.getElementById(
	"clear-key-btn",
) as HTMLButtonElement;
const cancelCaptureBtn = document.getElementById(
	"cancel-capture-btn",
) as HTMLButtonElement;
const confirmKeyBtn = document.getElementById(
	"confirm-key-btn",
) as HTMLButtonElement;

// 確認ダイアログ
const confirmModal = document.getElementById("confirm-modal") as HTMLElement;
const confirmTitle = document.getElementById("confirm-title") as HTMLElement;
const confirmMessage = document.getElementById(
	"confirm-message",
) as HTMLElement;
const confirmCancelBtn = document.getElementById(
	"confirm-cancel-btn",
) as HTMLButtonElement;
const confirmOkBtn = document.getElementById(
	"confirm-ok-btn",
) as HTMLButtonElement;

// 状態
let keybindings: AppConfig[] = [];
let selectedIndex: number = -1;
let hasChanges = false;
let currentKeyCaptureCallback: ((key: string) => void) | null = null;
let capturedKeys: string[] = [];
let confirmCallback: (() => void) | null = null;

// 初期化
async function init(): Promise<void> {
	await loadAndApplyTheme();
	setupSystemThemeListener();
	setupWindowFocusListener();
	await loadKeybindings();
	renderAppList();
	setupEventListeners();
}

// キーバインドデータを読み込む
async function loadKeybindings(): Promise<void> {
	try {
		keybindings = await invoke<AppConfig[]>("get_keybindings_raw");
	} catch (e) {
		console.error("Failed to load keybindings:", e);
		keybindings = [];
	}
}

// イベントリスナーの設定
function setupEventListeners(): void {
	closeBtn.addEventListener("click", handleClose);
	appSearchInput.addEventListener("input", handleAppSearch);
	addBtn.addEventListener("click", toggleAddMenu);
	addAppBtn.addEventListener("click", () => addNewItem("app"));
	addPlatformBtn.addEventListener("click", () => addNewItem("platform"));
	deleteAppBtn.addEventListener("click", handleDeleteApp);
	addKeybindingBtn.addEventListener("click", addKeybinding);
	resetBtn.addEventListener("click", handleReset);
	openFileBtn.addEventListener("click", handleOpenFile);
	cancelBtn.addEventListener("click", handleCancel);
	saveBtn.addEventListener("click", handleSave);

	// アプリ設定フォーム
	inputIcon.addEventListener("input", handleAppFormChange);
	inputName.addEventListener("input", handleAppFormChange);
	bindInput.addEventListener("keydown", handleBindInputKeydown);

	// プラットフォーム設定フォーム
	inputOs.addEventListener("change", handlePlatformFormChange);
	inputPlatformIcon.addEventListener("input", handlePlatformFormChange);

	// キーキャプチャモーダル
	clearKeyBtn.addEventListener("click", clearCapturedKey);
	cancelCaptureBtn.addEventListener("click", closeCaptureModal);
	confirmKeyBtn.addEventListener("click", confirmCapturedKey);

	// 確認ダイアログ
	confirmCancelBtn.addEventListener("click", closeConfirmModal);
	confirmOkBtn.addEventListener("click", handleConfirmOk);

	// メニュー外クリックで閉じる
	document.addEventListener("click", (e) => {
		if (
			!addBtn.contains(e.target as Node) &&
			!addMenu.contains(e.target as Node)
		) {
			addMenu.style.display = "none";
		}
	});

	// キーボードショートカット
	document.addEventListener("keydown", (e) => {
		if (e.ctrlKey && e.key === "s") {
			e.preventDefault();
			handleSave();
		} else if (e.key === "Escape") {
			if (keyCaptureModal.style.display !== "none") {
				closeCaptureModal();
			} else if (confirmModal.style.display !== "none") {
				closeConfirmModal();
			}
		}
	});
}

// アプリリストの描画
function renderAppList(filter = ""): void {
	const apps: AppConfig[] = [];
	const platforms: AppConfig[] = [];

	for (const config of keybindings) {
		if (config.os) {
			platforms.push(config);
		} else {
			apps.push(config);
		}
	}

	// フィルタリング
	const filterLower = filter.toLowerCase();
	const filteredApps = apps.filter((app) =>
		(app.name || "").toLowerCase().includes(filterLower),
	);

	// アプリリスト
	appListEl.innerHTML = "";
	for (let i = 0; i < keybindings.length; i++) {
		const config = keybindings[i];
		if (config.os) continue;
		if (!filteredApps.includes(config)) continue;

		const item = createAppItem(config, i);
		appListEl.appendChild(item);
	}

	if (filteredApps.length === 0) {
		appListEl.innerHTML = '<div class="empty-list">アプリがありません</div>';
	}

	// プラットフォームリスト
	platformListEl.innerHTML = "";
	for (let i = 0; i < keybindings.length; i++) {
		const config = keybindings[i];
		if (!config.os) continue;

		const item = createAppItem(config, i);
		platformListEl.appendChild(item);
	}

	if (platforms.length === 0) {
		platformListEl.innerHTML =
			'<div class="empty-list">プラットフォーム設定がありません</div>';
	}
}

// アプリアイテム要素の作成
function createAppItem(config: AppConfig, index: number): HTMLDivElement {
	const item = document.createElement("div");
	item.className = "app-item";
	if (index === selectedIndex) {
		item.classList.add("selected");
	}

	const icon =
		config.icon || (config.os ? getOsIcon(config.os) : DEFAULT_APP_ICON);
	const name = config.os ? getOsName(config.os) : config.name || UNNAMED_APP;

	item.innerHTML = `
		<div class="app-item-icon">${escapeHtml(icon)}</div>
		<div class="app-item-name">${escapeHtml(name)}</div>
	`;

	item.addEventListener("click", () => selectApp(index));

	return item;
}

// アプリ選択
function selectApp(index: number): void {
	selectedIndex = index;
	renderAppList(appSearchInput.value);
	showEditArea();
}

// 編集エリアの表示
function showEditArea(): void {
	if (selectedIndex < 0 || selectedIndex >= keybindings.length) {
		noSelectionEl.style.display = "flex";
		editAreaEl.style.display = "none";
		return;
	}

	noSelectionEl.style.display = "none";
	editAreaEl.style.display = "flex";

	const config = keybindings[selectedIndex];
	const isPlatform = !!config.os;

	if (isPlatform) {
		// プラットフォーム設定
		appFormEl.style.display = "none";
		platformFormEl.style.display = "block";
		editTypeEl.textContent = "プラットフォーム設定";

		const icon = config.icon || getOsIcon(config.os || OsType.Windows);
		editIconEl.textContent = icon;
		inputOs.value = config.os || OsType.Windows;
		inputPlatformIcon.value = config.icon || "";

		// 既存のOS設定を無効化
		updateOsSelectOptions();
	} else {
		// アプリ設定
		appFormEl.style.display = "block";
		platformFormEl.style.display = "none";
		editTypeEl.textContent = "アプリケーション設定";

		const icon = config.icon || DEFAULT_APP_ICON;
		editIconEl.textContent = icon;
		inputIcon.value = config.icon || "";
		inputName.value = config.name || "";

		// バインドタグを描画
		renderBindTags(config);
	}

	// キーバインドテーブルを描画
	renderKeybindingsTable(config);
}

// バインドタグの描画
function renderBindTags(config: AppConfig): void {
	// 既存のタグを削除
	const existingTags = bindTagsEl.querySelectorAll(".bind-tag");
	existingTags.forEach((tag) => {
		tag.remove();
	});

	// バインド値を取得
	const binds = getBinds(config);

	// タグを追加
	for (const bind of binds) {
		const tag = document.createElement("span");
		tag.className = "bind-tag";
		tag.innerHTML = `
			${escapeHtml(bind)}
			<button class="bind-tag-remove" title="削除">×</button>
		`;
		tag.querySelector(".bind-tag-remove")?.addEventListener("click", () => {
			removeBindTag(bind);
		});
		bindTagsEl.insertBefore(tag, bindInput);
	}
}

// バインド値を取得
function getBinds(config: AppConfig): string[] {
	if (!config.bind) return [];
	if (typeof config.bind === "string") return [config.bind];
	return config.bind;
}

// バインドタグを追加
function addBindTag(value: string): void {
	if (!value.trim()) return;

	const config = keybindings[selectedIndex];
	const binds = getBinds(config);

	if (binds.includes(value.trim())) return;

	binds.push(value.trim());
	config.bind = binds.length === 1 ? binds[0] : binds;

	markChanged();
	renderBindTags(config);
	bindInput.value = "";
}

// バインドタグを削除
function removeBindTag(value: string): void {
	const config = keybindings[selectedIndex];
	let binds = getBinds(config);

	binds = binds.filter((b) => b !== value);
	config.bind =
		binds.length === 0 ? undefined : binds.length === 1 ? binds[0] : binds;

	markChanged();
	renderBindTags(config);
}

// バインド入力のキーダウン処理
function handleBindInputKeydown(e: KeyboardEvent): void {
	if (e.key === "Enter") {
		e.preventDefault();
		addBindTag(bindInput.value);
	} else if (e.key === "Backspace" && bindInput.value === "") {
		// 入力が空の場合、最後のタグを削除
		const config = keybindings[selectedIndex];
		const binds = getBinds(config);
		if (binds.length > 0) {
			removeBindTag(binds[binds.length - 1]);
		}
	}
}

// OSセレクトのオプションを更新
function updateOsSelectOptions(): void {
	const existingOs = new Set<string>();
	for (let i = 0; i < keybindings.length; i++) {
		if (i !== selectedIndex && keybindings[i].os) {
			existingOs.add(keybindings[i].os as string);
		}
	}

	for (const option of inputOs.options) {
		option.disabled = existingOs.has(option.value);
	}
}

// キーバインドテーブルの描画
function renderKeybindingsTable(config: AppConfig): void {
	keybindingsTbody.innerHTML = "";

	if (!config.keybindings || config.keybindings.length === 0) {
		keybindingsTbody.innerHTML = `
			<tr>
				<td colspan="4">
					<div class="empty-keybindings">
						<div class="empty-keybindings-icon">⌨️</div>
						<p>キーバインドがありません</p>
					</div>
				</td>
			</tr>
		`;
		return;
	}

	for (let i = 0; i < config.keybindings.length; i++) {
		const kb = config.keybindings[i];
		const row = createKeybindingRow(kb, i);
		keybindingsTbody.appendChild(row);
	}
}

// キーバインド行の作成
function createKeybindingRow(
	kb: Keybinding,
	index: number,
): HTMLTableRowElement {
	const row = document.createElement("tr");

	// アクション列
	const actionTd = document.createElement("td");
	const actionInput = document.createElement("input");
	actionInput.type = "text";
	actionInput.value = kb.action;
	actionInput.placeholder = "アクション名";
	actionInput.addEventListener("input", () => {
		kb.action = actionInput.value;
		markChanged();
	});
	actionTd.appendChild(actionInput);

	// キー列
	const keyTd = document.createElement("td");
	const keyBtn = document.createElement("button");
	keyBtn.className = "key-input-btn";
	if (kb.key) {
		keyBtn.textContent = kb.key;
	} else {
		keyBtn.textContent = "クリックして入力";
		keyBtn.classList.add("placeholder");
	}
	keyBtn.addEventListener("click", () => {
		openCaptureModal((key) => {
			kb.key = key;
			keyBtn.textContent = key;
			keyBtn.classList.remove("placeholder");
			markChanged();
		});
	});
	keyTd.appendChild(keyBtn);

	// タグ列
	const tagsTd = document.createElement("td");
	const tagsInput = document.createElement("input");
	tagsInput.type = "text";
	tagsInput.value = (kb.tags || []).join(", ");
	tagsInput.placeholder = "タグ (カンマ区切り)";
	tagsInput.addEventListener("input", () => {
		kb.tags = tagsInput.value
			.split(",")
			.map((t) => t.trim())
			.filter((t) => t);
		markChanged();
	});
	tagsTd.appendChild(tagsInput);

	// 削除列
	const deleteTd = document.createElement("td");
	const deleteBtn = document.createElement("button");
	deleteBtn.className = "delete-row-btn";
	deleteBtn.innerHTML = `
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
			<line x1="18" y1="6" x2="6" y2="18"></line>
			<line x1="6" y1="6" x2="18" y2="18"></line>
		</svg>
	`;
	deleteBtn.addEventListener("click", () => {
		deleteKeybinding(index);
	});
	deleteTd.appendChild(deleteBtn);

	row.appendChild(actionTd);
	row.appendChild(keyTd);
	row.appendChild(tagsTd);
	row.appendChild(deleteTd);

	return row;
}

// キーバインドを追加
function addKeybinding(): void {
	if (selectedIndex < 0) return;

	const config = keybindings[selectedIndex];
	if (!config.keybindings) {
		config.keybindings = [];
	}

	config.keybindings.push({
		action: "",
		key: "",
		tags: [],
	});

	markChanged();
	renderKeybindingsTable(config);

	// 新しい行のアクション入力にフォーカス
	const lastRow = keybindingsTbody.lastElementChild;
	const actionInput = lastRow?.querySelector("input");
	actionInput?.focus();
}

// キーバインドを削除
function deleteKeybinding(index: number): void {
	const config = keybindings[selectedIndex];
	if (!config.keybindings) return;

	config.keybindings.splice(index, 1);
	markChanged();
	renderKeybindingsTable(config);
}

// キーキャプチャモーダルを開く
function openCaptureModal(callback: (key: string) => void): void {
	currentKeyCaptureCallback = callback;
	capturedKeys = [];
	sequenceModeCheckbox.checked = false;
	updateCapturedKeyDisplay();
	keyCaptureModal.style.display = "flex";

	// キー入力をリッスン
	document.addEventListener("keydown", handleKeyCaptureKeydown);
}

// キーキャプチャモーダルを閉じる
function closeCaptureModal(): void {
	keyCaptureModal.style.display = "none";
	currentKeyCaptureCallback = null;
	capturedKeys = [];
	document.removeEventListener("keydown", handleKeyCaptureKeydown);
}

// キーキャプチャのキーダウン処理
function handleKeyCaptureKeydown(e: KeyboardEvent): void {
	e.preventDefault();
	e.stopPropagation();

	// Escapeは無視（モーダルを閉じるため）
	if (e.key === "Escape") {
		closeCaptureModal();
		return;
	}

	// 修飾キーのみの場合は無視
	if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) {
		return;
	}

	const parts: string[] = [];
	if (e.ctrlKey) parts.push("Ctrl");
	if (e.shiftKey) parts.push("Shift");
	if (e.altKey) parts.push("Alt");
	if (e.metaKey) parts.push("Win");

	// キー名を正規化
	let keyName = e.key;
	if (keyName === " ") keyName = "Space";
	else if (keyName.length === 1) keyName = keyName.toUpperCase();

	parts.push(keyName);

	const keyCombo = parts.join(" + ");

	if (sequenceModeCheckbox.checked) {
		capturedKeys.push(keyCombo);
	} else {
		capturedKeys = [keyCombo];
	}

	updateCapturedKeyDisplay();
}

// キャプチャしたキーの表示を更新
function updateCapturedKeyDisplay(): void {
	if (capturedKeys.length === 0) {
		capturedKeyEl.innerHTML =
			'<span class="captured-key-placeholder">キーを押してください...</span>';
		return;
	}

	const html = capturedKeys
		.map((key) => {
			const parts = key.split(" + ");
			const keyBoxes = parts
				.map((p) => `<span class="captured-key-box">${escapeHtml(p)}</span>`)
				.join('<span class="captured-key-separator">+</span>');
			return `<span class="captured-key-group">${keyBoxes}</span>`;
		})
		.join('<span class="captured-key-separator">\u2192</span>');

	capturedKeyEl.innerHTML = `<div class="captured-key-display">${html}</div>`;
}

// キャプチャしたキーをクリア
function clearCapturedKey(): void {
	capturedKeys = [];
	updateCapturedKeyDisplay();
}

// キャプチャしたキーを確定
function confirmCapturedKey(): void {
	if (capturedKeys.length === 0) {
		closeCaptureModal();
		return;
	}

	const key = capturedKeys.join(" -> ");
	currentKeyCaptureCallback?.(key);
	closeCaptureModal();
}

// 確認ダイアログを表示
function showConfirmDialog(
	title: string,
	message: string,
	callback: () => void,
): void {
	confirmTitle.textContent = title;
	confirmMessage.textContent = message;
	confirmCallback = callback;
	confirmModal.style.display = "flex";
}

// 確認ダイアログを閉じる
function closeConfirmModal(): void {
	confirmModal.style.display = "none";
	confirmCallback = null;
}

// 確認OKボタンの処理
function handleConfirmOk(): void {
	confirmCallback?.();
	closeConfirmModal();
}

// アプリ検索
function handleAppSearch(): void {
	renderAppList(appSearchInput.value);
}

// 追加メニューの切り替え
function toggleAddMenu(): void {
	addMenu.style.display = addMenu.style.display === "none" ? "block" : "none";
}

// 新しいアイテムを追加
function addNewItem(type: "app" | "platform"): void {
	addMenu.style.display = "none";

	if (type === "app") {
		keybindings.push({
			icon: undefined,
			name: "新しいアプリ",
			bind: [],
			keybindings: [],
		});
	} else {
		// 既存のOS設定をチェック
		const existingOs = new Set<string>();
		for (const config of keybindings) {
			if (config.os) {
				existingOs.add(config.os);
			}
		}

		// 使用可能なOSを選択
		let os: OsTypeValue = OsType.Windows;
		if (existingOs.has(OsType.Windows) && !existingOs.has(OsType.Macos)) {
			os = OsType.Macos;
		} else if (existingOs.has(OsType.Windows) && existingOs.has(OsType.Macos)) {
			alert("すべてのプラットフォーム設定が既に存在します");
			return;
		}

		keybindings.push({
			os,
			icon: undefined,
			keybindings: [],
		});
	}

	markChanged();
	selectedIndex = keybindings.length - 1;
	renderAppList();
	showEditArea();
}

// アプリフォームの変更処理
function handleAppFormChange(): void {
	if (selectedIndex < 0) return;

	const config = keybindings[selectedIndex];
	config.icon = inputIcon.value || undefined;
	config.name = inputName.value;

	// アイコン表示を更新
	editIconEl.textContent = config.icon || DEFAULT_APP_ICON;

	markChanged();
	renderAppList(appSearchInput.value);
}

// プラットフォームフォームの変更処理
function handlePlatformFormChange(): void {
	if (selectedIndex < 0) return;

	const config = keybindings[selectedIndex];
	config.os = inputOs.value as OsTypeValue;
	config.icon = inputPlatformIcon.value || undefined;

	// アイコン表示を更新
	editIconEl.textContent =
		config.icon || getOsIcon(config.os || OsType.Windows);

	markChanged();
	renderAppList(appSearchInput.value);
}

// アプリ削除
function handleDeleteApp(): void {
	if (selectedIndex < 0) return;

	const config = keybindings[selectedIndex];
	const name = config.os ? getOsName(config.os) : config.name || "このアプリ";

	showConfirmDialog(
		"削除の確認",
		`「${name}」を削除しますか？この操作は取り消せません。`,
		() => {
			keybindings.splice(selectedIndex, 1);
			selectedIndex = -1;
			markChanged();
			renderAppList();
			showEditArea();
		},
	);
}

// 設定ファイルを開く
async function handleOpenFile(): Promise<void> {
	try {
		await invoke("open_config_file");
	} catch (e) {
		console.error("Failed to open config file:", e);
		alert("ファイルを開けませんでした");
	}
}

// リセット
function handleReset(): void {
	showConfirmDialog(
		"リセットの確認",
		"すべての設定をデフォルトに戻しますか？この操作は取り消せません。",
		async () => {
			try {
				keybindings = await invoke<AppConfig[]>("reset_keybindings");
				selectedIndex = -1;
				hasChanges = false;
				updateWindowTitle();
				renderAppList();
				showEditArea();
			} catch (e) {
				console.error("Failed to reset keybindings:", e);
				alert("リセットに失敗しました");
			}
		},
	);
}

// キャンセル
function handleCancel(): void {
	if (hasChanges) {
		showConfirmDialog(
			"変更の破棄",
			"未保存の変更があります。破棄しますか？",
			() => {
				closeWindow();
			},
		);
	} else {
		closeWindow();
	}
}

// 保存
async function handleSave(): Promise<void> {
	// バリデーション
	for (const config of keybindings) {
		if (!config.os && !config.name) {
			alert("アプリ名を入力してください");
			return;
		}
	}

	try {
		await invoke("save_keybindings", { config: keybindings });
		hasChanges = false;
		updateWindowTitle();
		closeWindow();
	} catch (e) {
		console.error("Failed to save keybindings:", e);
		alert("保存に失敗しました");
	}
}

// 閉じる
function handleClose(): void {
	if (hasChanges) {
		showConfirmDialog(
			"変更の破棄",
			"未保存の変更があります。破棄しますか？",
			() => {
				closeWindow();
			},
		);
	} else {
		closeWindow();
	}
}

// ウィンドウを閉じる
async function closeWindow(): Promise<void> {
	try {
		await invoke("close_keybindings_window");
	} catch (_e) {
		// フォールバック
		window.close();
	}
}

// 変更をマーク
function markChanged(): void {
	hasChanges = true;
	updateWindowTitle();
}

// ウィンドウタイトルを更新
function updateWindowTitle(): void {
	document.title = hasChanges ? "キーバインド設定 *" : "キーバインド設定";
}

// HTMLエスケープ
function escapeHtml(text: string): string {
	const div = document.createElement("div");
	div.textContent = text;
	return div.innerHTML;
}

// 初期化実行
document.addEventListener("DOMContentLoaded", init);
