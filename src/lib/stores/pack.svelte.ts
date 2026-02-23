import * as api from '$lib/utils/api';
import type { PackMeta } from '$lib/utils/api';

let packs = $state<PackMeta[]>([]);
let currentPack = $state<PackMeta | null>(null);
let loading = $state(false);

export const packStore = {
	get packs() { return packs; },
	get currentPack() { return currentPack; },
	get loading() { return loading; },

	async loadPacks() {
		loading = true;
		try {
			packs = await api.listPacks();
		} finally {
			loading = false;
		}
	},

	async loadPack(packId: string) {
		currentPack = await api.getPack(packId);
	},

	async createPack(name: string, description: string, versionId: string, iconPath?: string) {
		const pack = await api.createPack(name, description, versionId, iconPath);
		packs = [...packs, pack];
		return pack;
	},

	async updatePack(packId: string, name?: string, description?: string, iconPath?: string) {
		const updated = await api.updatePack(packId, name, description, iconPath);
		packs = packs.map((p) => (p.id === packId ? updated : p));
		if (currentPack?.id === packId) {
			currentPack = updated;
		}
		return updated;
	},

	async deletePack(packId: string) {
		await api.deletePack(packId);
		packs = packs.filter((p) => p.id !== packId);
		if (currentPack?.id === packId) {
			currentPack = null;
		}
	},

	async duplicatePack(packId: string, newName: string) {
		const dup = await api.duplicatePack(packId, newName);
		packs = [...packs, dup];
		return dup;
	},

	async changeVersion(packId: string, newVersionId: string) {
		const updated = await api.changePackVersion(packId, newVersionId);
		packs = packs.map((p) => (p.id === packId ? updated : p));
		if (currentPack?.id === packId) {
			currentPack = updated;
		}
		return updated;
	},

	async refreshCurrentPack() {
		if (currentPack) {
			currentPack = await api.getPack(currentPack.id);
		}
	},

	getProgress(pack: PackMeta, totalSounds: number) {
		return {
			recorded: pack.recordedSounds.length,
			total: totalSounds,
			percent: totalSounds > 0 ? Math.round((pack.recordedSounds.length / totalSounds) * 100) : 0,
		};
	},
};
