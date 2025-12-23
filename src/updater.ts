import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

/**
 * サイレントアップデートを実行
 * 新しいバージョンがあれば自動でダウンロード・インストール・再起動
 */
export async function checkAndInstallUpdate(): Promise<void> {
	try {
		const update = await check();
		if (update) {
			console.log(`Update available: ${update.version}`);
			await update.downloadAndInstall();
			await relaunch();
		}
	} catch (e) {
		// アップデートチェック失敗は無視（開発環境やオフライン時など）
		console.log("Update check skipped:", e);
	}
}
