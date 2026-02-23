import * as api from '$lib/utils/api';
import type { SoundNode } from '$lib/utils/api';

let soundTree = $state<SoundNode[]>([]);
let currentPath = $state<string[]>([]);
let selectedPaths = $state<Set<string>>(new Set());
let searchQuery = $state('');
let searchResults = $state<SoundNode[]>([]);
let loading = $state(false);

export const soundsStore = {
	get tree() { return soundTree; },
	get currentPath() { return currentPath; },
	get selectedPaths() { return selectedPaths; },
	get searchQuery() { return searchQuery; },
	get searchResults() { return searchResults; },
	get loading() { return loading; },

	get currentNodes(): SoundNode[] {
		let nodes = soundTree;
		for (const segment of currentPath) {
			const dir = nodes.find((n) => n.name === segment && n.nodeType === 'directory');
			if (dir?.children) {
				nodes = dir.children;
			} else {
				break;
			}
		}
		return nodes;
	},

	get breadcrumbs(): { name: string; path: string[] }[] {
		const crumbs = [{ name: 'sounds', path: [] as string[] }];
		for (let i = 0; i < currentPath.length; i++) {
			crumbs.push({
				name: currentPath[i],
				path: currentPath.slice(0, i + 1),
			});
		}
		return crumbs;
	},

	async loadTree(versionId: string, packId?: string) {
		loading = true;
		try {
			soundTree = await api.getSoundTree(versionId, packId);
			currentPath = [];
			selectedPaths = new Set();
			searchQuery = '';
			searchResults = [];
		} finally {
			loading = false;
		}
	},

	navigateTo(path: string[]) {
		currentPath = [...path];
		searchQuery = '';
		searchResults = [];
	},

	enterDirectory(dirName: string) {
		currentPath = [...currentPath, dirName];
	},

	toggleSelect(node: SoundNode) {
		const newSelected = new Set(selectedPaths);
		if (node.nodeType === 'file') {
			if (newSelected.has(node.path)) {
				newSelected.delete(node.path);
			} else {
				newSelected.add(node.path);
			}
		} else if (node.children) {
			const allFiles = collectFiles(node.children);
			const allSelected = allFiles.every((f) => newSelected.has(f.path));
			if (allSelected) {
				allFiles.forEach((f) => newSelected.delete(f.path));
			} else {
				allFiles.forEach((f) => newSelected.add(f.path));
			}
		}
		selectedPaths = newSelected;
	},

	clearSelection() {
		selectedPaths = new Set();
	},

	selectNodes(nodes: SoundNode[]) {
		const newSelected = new Set(selectedPaths);
		for (const node of nodes) {
			if (node.nodeType === 'file') {
				newSelected.add(node.path);
			} else if (node.children) {
				const allFiles = collectFiles(node.children);
				allFiles.forEach((f) => newSelected.add(f.path));
			}
		}
		selectedPaths = newSelected;
	},

	getSelectedSounds(): SoundNode[] {
		return flattenFiles(soundTree).filter((n) => selectedPaths.has(n.path));
	},

	setSearch(query: string) {
		searchQuery = query;
		if (query.trim()) {
			const allFiles = flattenFiles(soundTree);
			const q = query.toLowerCase();
			searchResults = allFiles.filter(
				(f) =>
					f.name.toLowerCase().includes(q) ||
					f.path.toLowerCase().includes(q) ||
					(f.soundEvent && f.soundEvent.toLowerCase().includes(q)),
			);
		} else {
			searchResults = [];
		}
	},

	navigateToFile(node: SoundNode) {
		// Navigate to the directory containing this file
		const pathParts = node.path
			.replace('minecraft/sounds/', '')
			.split('/');
		pathParts.pop(); // Remove filename
		currentPath = pathParts;
		searchQuery = '';
		searchResults = [];
		// Select the file
		selectedPaths = new Set([node.path]);
	},

	getTotalSoundCount(): number {
		return flattenFiles(soundTree).length;
	},
};

function collectFiles(nodes: SoundNode[]): SoundNode[] {
	const files: SoundNode[] = [];
	for (const node of nodes) {
		if (node.nodeType === 'file') {
			files.push(node);
		}
		if (node.children) {
			files.push(...collectFiles(node.children));
		}
	}
	return files;
}

function flattenFiles(nodes: SoundNode[]): SoundNode[] {
	return collectFiles(nodes);
}

// Check if all files in a directory are recorded
export function isDirectoryComplete(node: SoundNode, recordedSounds: string[]): boolean {
	if (node.nodeType === 'file') {
		return recordedSounds.includes(node.path);
	}
	if (!node.children || node.children.length === 0) return false;
	return node.children.every((child) => isDirectoryComplete(child, recordedSounds));
}

// Count files in a directory recursively
export function countFilesInDirectory(node: SoundNode): number {
	if (node.nodeType === 'file') return 1;
	if (!node.children) return 0;
	return node.children.reduce((sum, child) => sum + countFilesInDirectory(child), 0);
}
