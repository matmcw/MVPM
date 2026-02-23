import * as api from '$lib/utils/api';
import type { VersionEntry, DownloadProgress } from '$lib/utils/api';
import { listen } from '@tauri-apps/api/event';

let versions = $state<VersionEntry[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);
let downloadProgress = $state<DownloadProgress | null>(null);
let downloading = $state(false);

export const versionsStore = {
	get versions() { return versions; },
	get loading() { return loading; },
	get error() { return error; },
	get downloadProgress() { return downloadProgress; },
	get downloading() { return downloading; },

	async fetchVersions() {
		loading = true;
		error = null;
		try {
			versions = await api.fetchVersionManifest();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	},

	async isDownloaded(versionId: string): Promise<boolean> {
		return api.isVersionDownloaded(versionId);
	},

	async downloadVersion(versionId: string): Promise<boolean> {
		downloading = true;
		downloadProgress = null;

		const unlisten = await listen<DownloadProgress>('download-progress', (event) => {
			downloadProgress = event.payload;
		});

		try {
			await api.downloadVersionSounds(versionId);
			return true;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			return false;
		} finally {
			downloading = false;
			unlisten();
		}
	},

	releases() {
		return versions.filter((v) => v.versionType === 'release');
	},
};
