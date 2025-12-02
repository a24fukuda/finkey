// Tauri API (with fallback for development)
const invoke = window.__TAURI__?.tauri?.invoke || window.__TAURI__?.invoke || (async () => {});
const listen = window.__TAURI__?.event?.listen || (async () => () => {});

// DevToolsショートカットを無効化
document.addEventListener('keydown', (e) => {
  // Ctrl+Shift+J, Ctrl+Shift+I, F12, Ctrl+U を無効化
  if ((e.ctrlKey && e.shiftKey && (e.key === 'J' || e.key === 'j' || e.key === 'I' || e.key === 'i')) ||
      (e.ctrlKey && (e.key === 'U' || e.key === 'u')) ||
      e.key === 'F12') {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }
}, true);

// 右クリックメニューを無効化
document.addEventListener('contextmenu', (e) => {
  e.preventDefault();
});

// DOM要素
const activeAppNameEl = document.getElementById('active-app-name');
const searchInput = document.getElementById('search-input');
const resultsList = document.getElementById('results-list');
const noResults = document.getElementById('no-results');
const resultCount = document.getElementById('result-count');
const openConfigBtn = document.getElementById('open-config-btn');
const openAppsConfigBtn = document.getElementById('open-apps-config-btn');

// 状態
let currentPlatform = 'mac';
let selectedIndex = 0;
let filteredShortcuts = [];
let expandedIndex = -1;
let activeWindowInfo = null;
let matchedApps = [];
let shortcuts = [];

// アプリアイコンマッピング
const appIcons = {
  '*': '⌨️',
  'VS Code': '💻',
  'Cursor': '💻',
  'Chrome': '🌐',
  'Edge': '🌐',
  'Firefox': '🌐',
  'Safari': '🌐',
  'Brave': '🌐',
  'Slack': '💬',
  'Zoom': '📹',
  'Excel': '📊',
  'エクスプローラー': '📁',
  'Finder': '📁',
  'Windows Terminal': '⬛',
  'Terminal': '⬛',
  'PowerShell': '⬛',
  'コマンドプロンプト': '⬛'
};

// 初期化
async function init() {
  // プラットフォーム検出
  try {
    const platform = await invoke('get_platform');
    currentPlatform = platform === 'darwin' ? 'mac' : 'windows';
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

  // イベントリスナー
  searchInput.addEventListener('input', handleTextSearch);
  searchInput.addEventListener('keydown', handleKeydown);
  openConfigBtn.addEventListener('click', openConfigFile);
  openAppsConfigBtn.addEventListener('click', openAppsConfigFile);

  // Tauriイベントリスナー（アクティブウィンドウ情報を受け取る）
  try {
    await listen('window-shown', async (event) => {
      activeWindowInfo = event.payload || null;

      // バックエンドでアプリをマッチング
      try {
        matchedApps = await invoke('get_matched_apps', { info: activeWindowInfo });
      } catch (e) {
        console.log('Failed to get matched apps:', e);
        matchedApps = [];
      }

      // UIにアプリ名を表示
      let displayText = '-';
      if (matchedApps.length > 0) {
        displayText = matchedApps.map(app => app.display).join(', ');
      } else if (activeWindowInfo) {
        displayText = activeWindowInfo.process || '-';
      }
      activeAppNameEl.textContent = displayText;

      // 状態をリセット
      selectedIndex = 0;
      expandedIndex = -1;
      searchInput.value = '';
      searchInput.focus();
      searchInput.select();

      filterAndDisplay();
    });
  } catch (e) {
    // イベントリスナー登録に失敗
  }
}

// テキスト検索処理
function handleTextSearch() {
  selectedIndex = 0;
  expandedIndex = -1;
  filterAndDisplay();
}

// キーボードナビゲーション
async function handleKeydown(e) {
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

// ショートカット設定ファイルを開く
async function openConfigFile() {
  try {
    await invoke('open_config_file');
  } catch (e) {
    console.log('Failed to open config file:', e);
  }
}

// アプリ設定ファイルを開く
async function openAppsConfigFile() {
  try {
    await invoke('open_apps_config_file');
  } catch (e) {
    console.log('Failed to open apps config file:', e);
  }
}

// フィルタリングと表示
function filterAndDisplay() {
  filterByText();
  displayResults();
}

// テキストでフィルタリング
function filterByText() {
  const query = searchInput.value.toLowerCase().trim();

  filteredShortcuts = shortcuts.filter(shortcut => {
    // 検索クエリがない場合は全て表示
    if (!query) {
      return true;
    }

    // 検索マッチング（app, action, key, description, tags）
    const searchTargets = [
      shortcut.app,
      shortcut.action,
      shortcut.description,
      shortcut.key,
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

  // アプリ名の一致
  if (shortcut.app !== '*') {
    if (shortcut.app.toLowerCase() === q) score += 80;
    else if (shortcut.app.toLowerCase().startsWith(q)) score += 60;
    else if (shortcut.app.toLowerCase().includes(q)) score += 40;
  }

  // タグの一致（ローマ字検索はタグに含まれる）
  shortcut.tags.forEach(tag => {
    if (tag.toLowerCase() === q) score += 45;
    else if (tag.toLowerCase().startsWith(q)) score += 30;
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

  const icon = appIcons[shortcut.app] || '⌨️';
  const displayKey = shortcut.key;
  // アプリ名表示（"*"は「共通」と表示）
  const appLabel = shortcut.app === '*' ? '共通' : shortcut.app;

  // ハイライト処理
  const query = searchInput.value.toLowerCase().trim();
  const highlightedAction = highlightText(shortcut.action, query);
  const highlightedDesc = highlightText(shortcut.description, query);

  let html = `
    <div class="result-icon">${icon}</div>
    <div class="result-content">
      <div class="result-action">${highlightedAction}</div>
      <div class="result-description">${highlightedDesc}</div>
      <span class="result-category">${appLabel}</span>
    </div>
    <div class="result-shortcut">
      <span class="shortcut-key ${currentPlatform}">${escapeHtml(displayKey)}</span>
    </div>
  `;

  // 展開時の詳細
  if (index === expandedIndex) {
    html += `
      <div class="result-details">
        <div class="result-details-row">
          <span class="detail-label">キー:</span>
          <span class="shortcut-key ${currentPlatform}">${escapeHtml(shortcut.key)}</span>
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
