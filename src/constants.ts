// アプリケーション全体で使用する定数

// デフォルトアイコン
export const DEFAULT_APP_ICON = "\u{1F4CC}"; // 📌
export const WINDOWS_ICON = "\u{1FA9F}"; // 🪟
export const MACOS_ICON = "\u{1F34E}"; // 🍎

// OS種別
export const OsType = {
	Windows: "windows",
	Macos: "macos",
} as const;

export type OsTypeValue = (typeof OsType)[keyof typeof OsType];

// OS表示名
export const WINDOWS_NAME = "Windows";
export const MACOS_NAME = "macOS";

// プレースホルダテキスト
export const UNNAMED_APP = "無名のアプリ";

// OS種別に応じたアイコンを取得
export function getOsIcon(os: OsTypeValue): string {
	return os === OsType.Windows ? WINDOWS_ICON : MACOS_ICON;
}

// OS種別に応じた表示名を取得
export function getOsName(os: OsTypeValue): string {
	return os === OsType.Windows ? WINDOWS_NAME : MACOS_NAME;
}
