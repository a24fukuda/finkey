// Tauri API (with fallback for development)
const invoke = window.__TAURI__?.tauri?.invoke || window.__TAURI__?.invoke || (async () => {});
const listen = window.__TAURI__?.event?.listen || (async () => () => {});

// DOM要素
const activeAppNameEl = document.getElementById('active-app-name');
const activeAppNameTextEl = document.getElementById('active-app-name-text');
const keyInputContainer = document.getElementById('key-input-container');
const keyInputDisplay = document.getElementById('key-input-display');
const searchContainer = document.getElementById('search-container');
const searchInput = document.getElementById('search-input');
const resultsList = document.getElementById('results-list');
const noResults = document.getElementById('no-results');
const resultCount = document.getElementById('result-count');
const modeToggle = document.getElementById('mode-toggle');
const modeToggleText = document.getElementById('mode-toggle-text');
const openConfigBtn = document.getElementById('open-config-btn');

// 状態
let currentPlatform = 'mac';
let selectedIndex = 0;
let filteredShortcuts = [];
let expandedIndex = -1;
let searchMode = 'key'; // 'key' または 'text'
let pressedKeys = new Set();
let pressedKeyDetails = new Map(); // key -> { code, display }
let activeAppName = null; // アクティブなアプリ名
let activeAppCategory = null; // アクティブなアプリに対応するカテゴリ
let shortcuts = []; // バックエンドから読み込むショートカットデータ

// プロセス名からカテゴリへのマッピング
const appToCategoryMap = {
  // ブラウザ
  'chrome': 'ブラウザ',
  'Google Chrome': 'ブラウザ',
  'msedge': 'ブラウザ',
  'Microsoft Edge': 'ブラウザ',
  'firefox': 'ブラウザ',
  'Firefox': 'ブラウザ',
  'Safari': 'ブラウザ',
  'brave': 'ブラウザ',
  'Brave Browser': 'ブラウザ',
  // VS Code
  'Code': 'VS Code',
  'code': 'VS Code',
  'Visual Studio Code': 'VS Code',
  'Cursor': 'VS Code',
  // ファイルマネージャー
  'explorer': 'Finder / エクスプローラー',
  'Explorer': 'Finder / エクスプローラー',
  'Finder': 'Finder / エクスプローラー',
  // Slack
  'slack': 'Slack',
  'Slack': 'Slack',
  // Excel / スプレッドシート
  'EXCEL': 'Excel / スプレッドシート',
  'excel': 'Excel / スプレッドシート',
  // ターミナル
  'WindowsTerminal': 'ターミナル',
  'Windows Terminal': 'ターミナル',
  'cmd': 'ターミナル',
  'powershell': 'ターミナル',
  'Terminal': 'ターミナル',
  // Zoom
  'Zoom': 'Zoom',
  'zoom': 'Zoom',
};

// カテゴリアイコンマッピング
const categoryIcons = {
  '一般': '⌨️',
  'テキスト編集': '✏️',
  'ブラウザ': '🌐',
  'システム（Mac）': '🍎',
  'システム（Windows）': '🪟',
  'VS Code': '💻',
  'Finder / エクスプローラー': '📁',
  'Slack': '💬',
  'Excel / スプレッドシート': '📊',
  'ターミナル': '⬛',
  'Zoom': '📹'
};

// キー名の正規化マッピング
const keyDisplayMap = {
  // 修飾キー
  'Control': 'Ctrl',
  'Meta': '⌘',
  'Alt': currentPlatform === 'mac' ? 'Option' : 'Alt',
  'Shift': 'Shift',
  // 矢印キー
  'ArrowUp': '↑',
  'ArrowDown': '↓',
  'ArrowLeft': '←',
  'ArrowRight': '→',
  // 特殊キー
  'Escape': 'Esc',
  'Enter': 'Enter',
  'Backspace': 'Backspace',
  'Delete': 'Delete',
  'Tab': 'Tab',
  'Space': 'Space',
  ' ': 'Space',
  'Home': 'Home',
  'End': 'End',
  'PageUp': 'Page Up',
  'PageDown': 'Page Down',
  'Insert': 'Insert',
  // ファンクションキー
  'F1': 'F1', 'F2': 'F2', 'F3': 'F3', 'F4': 'F4',
  'F5': 'F5', 'F6': 'F6', 'F7': 'F7', 'F8': 'F8',
  'F9': 'F9', 'F10': 'F10', 'F11': 'F11', 'F12': 'F12',
};

// キーの優先順位（修飾キーを先に表示）
const keyOrder = {
  'Control': 1, 'Ctrl': 1,
  'Meta': 2, '⌘': 2, 'Win': 2,
  'Alt': 3, 'Option': 3,
  'Shift': 4,
};

// 初期化
async function init() {
  // プラットフォーム検出
  try {
    const platform = await invoke('get_platform');
    if (platform === 'darwin') {
      setPlatform('mac');
    } else {
      setPlatform('windows');
    }
  } catch (e) {
    console.log('Platform detection failed, defaulting to mac');
  }

  // ショートカットデータをバックエンドから読み込む
  try {
    shortcuts = await invoke('get_shortcuts');
  } catch (e) {
    console.log('Failed to load shortcuts from backend, using empty list');
    shortcuts = [];
  }

  // 初期表示
  filterAndDisplay();

  // イベントリスナー（テキスト検索用）
  searchInput.addEventListener('input', handleTextSearch);
  searchInput.addEventListener('keydown', handleTextModeKeydown);

  // モード切り替え
  modeToggle.addEventListener('click', () => setSearchMode('text'));
  modeToggleText.addEventListener('click', () => setSearchMode('key'));

  // 設定ファイルを開くボタン
  openConfigBtn.addEventListener('click', openConfigFile);

  // キー入力モードのイベントリスナー
  document.addEventListener('keydown', handleGlobalKeydown);
  document.addEventListener('keyup', handleGlobalKeyup);

  // ウィンドウがフォーカスを失った時にキーをリセット
  window.addEventListener('blur', resetPressedKeys);

  // Tauriイベントリスナー（アクティブアプリ名を受け取る）
  try {
    await listen('window-shown', (event) => {
      // アクティブアプリ名を取得
      activeAppName = event.payload || null;
      activeAppCategory = activeAppName ? (appToCategoryMap[activeAppName] || null) : null;

      // UIにアプリ名を表示（両方のモードで表示）
      const displayText = activeAppName
        ? (activeAppCategory ? `${activeAppName}` : `${activeAppName}`)
        : '-';
      activeAppNameEl.textContent = displayText;
      if (activeAppNameTextEl) {
        activeAppNameTextEl.textContent = displayText;
      }

    // 状態をリセット
    selectedIndex = 0;
    expandedIndex = -1;
    resetPressedKeys();

    if (searchMode === 'text') {
      searchInput.value = '';
      searchInput.focus();
      searchInput.select();
    }

    filterAndDisplay();
    });
  } catch (e) {
    // イベントリスナー登録に失敗
  }
}

// 検索モード切り替え
function setSearchMode(mode) {
  searchMode = mode;
  if (mode === 'key') {
    keyInputContainer.style.display = 'flex';
    searchContainer.style.display = 'none';
    resetPressedKeys();
    filterAndDisplay();
  } else {
    keyInputContainer.style.display = 'none';
    searchContainer.style.display = 'flex';
    searchInput.focus();
    filterAndDisplay();
  }
}

// キー入力リセット
function resetPressedKeys() {
  pressedKeys.clear();
  pressedKeyDetails.clear();
  updateKeyDisplay();
  if (searchMode === 'key') {
    filterAndDisplay();
  }
}

// グローバルキーダウンハンドラ
function handleGlobalKeydown(e) {
  if (searchMode !== 'key') return;

  // Escapeは特別処理
  if (e.key === 'Escape') {
    e.preventDefault();
    hideWindow();
    return;
  }

  // 上下キーでリスト操作
  if (e.key === 'ArrowDown' && !hasModifierKeys(e)) {
    e.preventDefault();
    if (selectedIndex < filteredShortcuts.length - 1) {
      selectedIndex++;
      updateSelection();
      scrollToSelected();
    }
    return;
  }

  if (e.key === 'ArrowUp' && !hasModifierKeys(e)) {
    e.preventDefault();
    if (selectedIndex > 0) {
      selectedIndex--;
      updateSelection();
      scrollToSelected();
    }
    return;
  }

  // Enterで展開
  if (e.key === 'Enter' && !hasModifierKeys(e)) {
    e.preventDefault();
    toggleExpand(selectedIndex);
    return;
  }

  e.preventDefault();

  const keyId = getKeyId(e);
  const displayName = getKeyDisplayName(e);

  if (!pressedKeys.has(keyId)) {
    pressedKeys.add(keyId);
    pressedKeyDetails.set(keyId, { code: e.code, display: displayName, key: e.key });
    updateKeyDisplay();
    filterByPressedKeys();
  }
}

// 修飾キーが押されているか
function hasModifierKeys(e) {
  return e.ctrlKey || e.metaKey || e.altKey;
}

// グローバルキーアップハンドラ
function handleGlobalKeyup(e) {
  if (searchMode !== 'key') return;

  const keyId = getKeyId(e);

  if (pressedKeys.has(keyId)) {
    pressedKeys.delete(keyId);
    pressedKeyDetails.delete(keyId);
    updateKeyDisplay();
    filterByPressedKeys();
  }
}

// キーの一意識別子を取得
function getKeyId(e) {
  // 修飾キーはkey名で識別
  if (['Control', 'Meta', 'Alt', 'Shift'].includes(e.key)) {
    return e.key;
  }
  // その他のキーはcodeで識別（左右のキーを区別しないため）
  return e.code || e.key;
}

// キーの表示名を取得
function getKeyDisplayName(e) {
  const key = e.key;

  // マッピングにあればそれを使用
  if (keyDisplayMap[key]) {
    // Altキーはプラットフォームに応じて変更
    if (key === 'Alt') {
      return currentPlatform === 'mac' ? 'Option' : 'Alt';
    }
    return keyDisplayMap[key];
  }

  // 1文字の場合は大文字に
  if (key.length === 1) {
    return key.toUpperCase();
  }

  return key;
}

// キー表示を更新
function updateKeyDisplay() {
  if (pressedKeys.size === 0) {
    keyInputDisplay.innerHTML = '<span class="key-placeholder">キーを押してください...</span>';
    return;
  }

  // キーをソートして表示
  const sortedKeys = Array.from(pressedKeyDetails.values())
    .sort((a, b) => {
      const orderA = keyOrder[a.display] || 100;
      const orderB = keyOrder[b.display] || 100;
      return orderA - orderB;
    });

  keyInputDisplay.innerHTML = sortedKeys
    .map(k => `<span class="pressed-key">${escapeHtml(k.display)}</span>`)
    .join('<span class="key-separator">+</span>');
}

// 押されているキーでフィルタリング
function filterByPressedKeys() {
  selectedIndex = 0;
  expandedIndex = -1;
  filterAndDisplay();
}

// プラットフォーム設定（自動検出のみ）
function setPlatform(platform) {
  currentPlatform = platform;
  filterAndDisplay();
}

// テキスト検索処理
function handleTextSearch() {
  selectedIndex = 0;
  expandedIndex = -1;
  filterAndDisplay();
}

// テキストモードでのキーボードナビゲーション
async function handleTextModeKeydown(e) {
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      if (selectedIndex < filteredShortcuts.length - 1) {
        selectedIndex++;
        updateSelection();
        scrollToSelected();
      }
      break;
    case 'ArrowUp':
      e.preventDefault();
      if (selectedIndex > 0) {
        selectedIndex--;
        updateSelection();
        scrollToSelected();
      }
      break;
    case 'Enter':
      e.preventDefault();
      toggleExpand(selectedIndex);
      break;
    case 'Escape':
      e.preventDefault();
      hideWindow();
      break;
  }
}

// ウィンドウを隠す
async function hideWindow() {
  try {
    await invoke('hide_main_window');
  } catch (e) {
    console.log('Hide window failed');
  }
}

// 設定ファイルを開く
async function openConfigFile() {
  try {
    await invoke('open_config_file');
  } catch (e) {
    console.log('Failed to open config file:', e);
  }
}

// フィルタリングと表示
function filterAndDisplay() {
  if (searchMode === 'key') {
    filterByKeys();
  } else {
    filterByText();
  }
  displayResults();
}

// キー入力でフィルタリング
function filterByKeys() {
  if (pressedKeys.size === 0) {
    // キーが押されていない場合は全て表示
    filteredShortcuts = shortcuts.filter(shortcut => {
      const platformKey = currentPlatform === 'mac' ? shortcut.mac : shortcut.windows;
      return platformKey !== '-';
    });
    return;
  }

  // 押されているキーの表示名を取得
  const pressedKeyDisplays = Array.from(pressedKeyDetails.values())
    .map(k => k.display.toLowerCase());

  filteredShortcuts = shortcuts.filter(shortcut => {
    // プラットフォームフィルター
    const platformKey = currentPlatform === 'mac' ? shortcut.mac : shortcut.windows;
    if (platformKey === '-') {
      return false;
    }

    // ショートカットキーのマッチング
    return matchShortcutKeys(platformKey, pressedKeyDisplays);
  });

  // マッチ度でソート
  filteredShortcuts.sort((a, b) => {
    const aKey = currentPlatform === 'mac' ? a.mac : a.windows;
    const bKey = currentPlatform === 'mac' ? b.mac : b.windows;
    const aScore = getKeyMatchScore(aKey, pressedKeyDisplays);
    const bScore = getKeyMatchScore(bKey, pressedKeyDisplays);
    return bScore - aScore;
  });
}

// ショートカットキーのマッチング
function matchShortcutKeys(shortcutKey, pressedKeyDisplays) {
  const normalizedShortcut = normalizeShortcutString(shortcutKey);

  // 全ての押されているキーがショートカットに含まれているか
  return pressedKeyDisplays.every(pressed => {
    return normalizedShortcut.includes(pressed);
  });
}

// ショートカット文字列を正規化
function normalizeShortcutString(str) {
  return str
    .toLowerCase()
    .replace(/⌘/g, 'ctrl meta ⌘')
    .replace(/command/gi, 'ctrl meta ⌘')
    .replace(/cmd/gi, 'ctrl meta ⌘')
    .replace(/control/gi, 'ctrl')
    .replace(/option/gi, 'alt option')
    .replace(/\+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

// キーマッチスコアを計算
function getKeyMatchScore(shortcutKey, pressedKeyDisplays) {
  const normalizedShortcut = normalizeShortcutString(shortcutKey);
  const shortcutParts = normalizedShortcut.split(' ').filter(p => p.length > 0);

  let score = 0;

  // マッチしたキーの数
  pressedKeyDisplays.forEach(pressed => {
    if (normalizedShortcut.includes(pressed)) {
      score += 10;
    }
  });

  // 完全一致ボーナス（押されているキーの数とショートカットのキー数が同じ）
  // ショートカットのユニークなキー部分を推定
  const uniqueShortcutKeys = new Set(shortcutParts.filter(p =>
    !['/', '|', 'または', 'or'].includes(p)
  ));

  if (pressedKeyDisplays.length === uniqueShortcutKeys.size) {
    score += 50;
  }

  return score;
}

// テキストでフィルタリング
function filterByText() {
  const query = searchInput.value.toLowerCase().trim();

  filteredShortcuts = shortcuts.filter(shortcut => {
    // プラットフォームフィルター
    const platformKey = currentPlatform === 'mac' ? shortcut.mac : shortcut.windows;
    if (platformKey === '-') {
      return false;
    }

    // 検索クエリがない場合は全て表示
    if (!query) {
      return true;
    }

    // 検索マッチング
    const searchTargets = [
      shortcut.action,
      shortcut.description,
      shortcut.mac,
      shortcut.windows,
      shortcut.category,
      ...shortcut.tags
    ].map(s => s.toLowerCase());

    return searchTargets.some(target => target.includes(query));
  });

  // 検索クエリがある場合は関連度でソート
  if (query) {
    filteredShortcuts.sort((a, b) => {
      const aScore = getTextRelevanceScore(a, query);
      const bScore = getTextRelevanceScore(b, query);
      return bScore - aScore;
    });
  }
}

// テキスト関連度スコア計算
function getTextRelevanceScore(shortcut, query) {
  let score = 0;
  const q = query.toLowerCase();

  // アクション名の完全一致
  if (shortcut.action.toLowerCase() === q) score += 100;
  // アクション名の先頭一致
  else if (shortcut.action.toLowerCase().startsWith(q)) score += 70;
  // アクション名の部分一致
  else if (shortcut.action.toLowerCase().includes(q)) score += 50;

  // タグの一致
  shortcut.tags.forEach(tag => {
    if (tag.toLowerCase() === q) score += 40;
    else if (tag.toLowerCase().startsWith(q)) score += 25;
    else if (tag.toLowerCase().includes(q)) score += 15;
  });

  // 説明の一致
  if (shortcut.description.toLowerCase().includes(q)) score += 10;

  return score;
}

// 結果表示
function displayResults() {
  resultsList.innerHTML = '';

  if (filteredShortcuts.length === 0) {
    noResults.style.display = 'block';
    resultCount.textContent = '';
    return;
  }

  noResults.style.display = 'none';
  resultCount.textContent = `${filteredShortcuts.length}件`;

  // DocumentFragmentで一括追加（パフォーマンス向上）
  const fragment = document.createDocumentFragment();

  filteredShortcuts.forEach((shortcut, index) => {
    const item = createResultItem(shortcut, index);
    fragment.appendChild(item);
  });

  resultsList.appendChild(fragment);
  updateSelection();
}

// 結果アイテム作成
function createResultItem(shortcut, index) {
  const item = document.createElement('div');
  item.className = 'result-item';
  if (index === selectedIndex) item.classList.add('selected');
  if (index === expandedIndex) item.classList.add('expanded');
  item.dataset.index = index;

  const icon = categoryIcons[shortcut.category] || '⌨️';
  const displayKey = currentPlatform === 'mac' ? shortcut.mac : shortcut.windows;

  // ハイライト処理
  const query = searchMode === 'text' ? searchInput.value.toLowerCase().trim() : '';
  const highlightedAction = highlightText(shortcut.action, query);
  const highlightedDesc = highlightText(shortcut.description, query);

  // キーのハイライト（キー入力モードの場合）
  let highlightedKey = escapeHtml(displayKey);
  if (searchMode === 'key' && pressedKeys.size > 0) {
    highlightedKey = highlightKeyParts(displayKey);
  }

  let html = `
    <div class="result-icon">${icon}</div>
    <div class="result-content">
      <div class="result-action">${highlightedAction}</div>
      <div class="result-description">${highlightedDesc}</div>
      <span class="result-category">${shortcut.category}</span>
    </div>
    <div class="result-shortcut">
      <span class="shortcut-key ${currentPlatform}">${highlightedKey}</span>
    </div>
  `;

  // 展開時の詳細
  if (index === expandedIndex) {
    html += `
      <div class="result-details">
        <div class="result-details-row">
          <span class="detail-label">Mac:</span>
          <span class="shortcut-key mac">${escapeHtml(shortcut.mac)}</span>
        </div>
        <div class="result-details-row">
          <span class="detail-label">Windows:</span>
          <span class="shortcut-key win">${escapeHtml(shortcut.windows)}</span>
        </div>
        <div class="result-details-row" style="margin-top: 8px;">
          <span class="detail-label">タグ:</span>
          <span class="detail-value">${shortcut.tags.join(', ')}</span>
        </div>
      </div>
    `;
  }

  item.innerHTML = html;

  item.addEventListener('click', () => {
    selectedIndex = index;
    updateSelection();
    toggleExpand(index);
  });

  return item;
}

// キー部分をハイライト
function highlightKeyParts(keyString) {
  const pressedKeyDisplays = Array.from(pressedKeyDetails.values())
    .map(k => k.display);

  let result = escapeHtml(keyString);

  pressedKeyDisplays.forEach(pressed => {
    const escapedPressed = escapeHtml(pressed);
    const regex = new RegExp(`(${escapeRegExp(escapedPressed)})`, 'gi');
    result = result.replace(regex, '<span class="key-highlight">$1</span>');
  });

  return result;
}

// テキストハイライト
function highlightText(text, query) {
  if (!query) return escapeHtml(text);

  const escaped = escapeHtml(text);
  const escapedQuery = escapeHtml(query);
  const regex = new RegExp(`(${escapeRegExp(escapedQuery)})`, 'gi');
  return escaped.replace(regex, '<span class="highlight">$1</span>');
}

// HTMLエスケープ
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// 正規表現エスケープ
function escapeRegExp(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

// 選択状態更新
function updateSelection() {
  document.querySelectorAll('.result-item').forEach((item, index) => {
    item.classList.toggle('selected', index === selectedIndex);
  });
}

// 選択アイテムにスクロール
function scrollToSelected() {
  const selected = document.querySelector('.result-item.selected');
  if (selected) {
    selected.scrollIntoView({ block: 'nearest', behavior: 'auto' });
  }
}

// 展開/折りたたみ
function toggleExpand(index) {
  if (expandedIndex === index) {
    expandedIndex = -1;
  } else {
    expandedIndex = index;
  }
  displayResults();
}

// 初期化実行
document.addEventListener('DOMContentLoaded', init);
