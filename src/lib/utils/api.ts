import { invoke } from '@tauri-apps/api/core';

// --- Types ---

export interface Settings {
	record_key: string;
	microphone_id: string | null;
	auto_play_original: boolean;
	single_recording_mode: boolean;
	packs_folder: string | null;
	theme: string;
	window_width: number | null;
	window_height: number | null;
	window_x: number | null;
	window_y: number | null;
}

export interface VersionEntry {
	id: string;
	versionType: string;
	url: string;
	time: string;
	releaseTime: string;
}

export interface DownloadStatus {
	totalFiles: number;
	downloadedFiles: number;
	totalBytes: number;
	downloadedBytes: number;
	complete: boolean;
	failedFiles: string[];
}

export interface DownloadProgress {
	versionId: string;
	downloadedFiles: number;
	totalFiles: number;
	downloadedBytes: number;
	totalBytes: number;
	currentFile: string;
	status: string;
}

export interface SoundNode {
	name: string;
	path: string;
	nodeType: 'directory' | 'file';
	children?: SoundNode[];
	hash?: string;
	size?: number;
	isLongSound: boolean;
	soundEvent?: string;
	variants?: string[];
}

export interface PackMeta {
	id: string;
	name: string;
	description: string;
	versionId: string;
	packFormat: number;
	hasIcon: boolean;
	iconPath?: string;
	recordedSounds: string[];
	createdAt: string;
}

// --- Settings ---

export async function getSettings(): Promise<Settings> {
	return invoke('get_settings');
}

export async function saveSettings(settings: Settings): Promise<void> {
	return invoke('save_settings', { settings });
}

// --- Mojang ---

export async function fetchVersionManifest(): Promise<VersionEntry[]> {
	return invoke('fetch_version_manifest');
}

export async function isVersionDownloaded(versionId: string): Promise<boolean> {
	return invoke('is_version_downloaded', { versionId });
}

export async function getDownloadStatus(versionId: string): Promise<DownloadStatus> {
	return invoke('get_download_status', { versionId });
}

export async function downloadVersionSounds(versionId: string): Promise<void> {
	return invoke('download_version_sounds', { versionId });
}

export async function getSoundTree(versionId: string, packId?: string): Promise<SoundNode[]> {
	return invoke('get_sound_tree', { versionId, packId: packId ?? null });
}

export async function getOriginalSoundPath(versionId: string, soundPath: string): Promise<string> {
	return invoke('get_original_sound_path', { versionId, soundPath });
}

// --- Packs ---

export async function listPacks(): Promise<PackMeta[]> {
	return invoke('list_packs');
}

export async function createPack(
	name: string,
	description: string,
	versionId: string,
	iconPath?: string,
): Promise<PackMeta> {
	return invoke('create_pack', {
		name,
		description,
		versionId,
		iconPath: iconPath ?? null,
	});
}

export async function getPack(packId: string): Promise<PackMeta> {
	return invoke('get_pack', { packId });
}

export async function updatePack(
	packId: string,
	name?: string,
	description?: string,
	iconPath?: string,
): Promise<PackMeta> {
	return invoke('update_pack', {
		packId,
		name: name ?? null,
		description: description ?? null,
		iconPath: iconPath ?? null,
	});
}

export async function deletePack(packId: string): Promise<void> {
	return invoke('delete_pack', { packId });
}

export async function duplicatePack(packId: string, newName: string): Promise<PackMeta> {
	return invoke('duplicate_pack', { packId, newName });
}

export async function changePackVersion(packId: string, newVersionId: string): Promise<PackMeta> {
	return invoke('change_pack_version', { packId, newVersionId });
}

export async function getPackSoundPath(packId: string, soundPath: string): Promise<string | null> {
	return invoke('get_pack_sound_path', { packId, soundPath });
}

// --- Recording ---

export async function saveRecording(
	packId: string,
	soundPath: string,
	wavData: number[],
	singleMode: boolean,
	variantPaths?: string[],
): Promise<void> {
	return invoke('save_recording', {
		packId,
		soundPath,
		wavData,
		singleMode,
		variantPaths: variantPaths ?? null,
	});
}

export async function deleteRecording(packId: string, soundPath: string): Promise<void> {
	return invoke('delete_recording', { packId, soundPath });
}
