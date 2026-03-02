<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { packStore } from '$lib/stores/pack.svelte';
	import { soundsStore } from '$lib/stores/sounds.svelte';
	import { recordingStore } from '$lib/stores/recording.svelte';
	import TileGrid from '$lib/components/TileGrid.svelte';
	import Breadcrumb from '$lib/components/Breadcrumb.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import WarningDialog from '$lib/components/WarningDialog.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import type { SoundNode } from '$lib/utils/api';

	const packId = $derived(page.params.id);
	let showRerecordWarning = $state(false);

	onMount(async () => {
		await packStore.loadPack(packId);
		if (packStore.currentPack) {
			await soundsStore.loadTree(packStore.currentPack.versionId, packId);
		}
	});

	const recorded = $derived(packStore.currentPack?.recordedSounds ?? []);
	const totalSounds = $derived(
		settingsStore.singleRecordingMode
			? soundsStore.getDedupedSoundCount()
			: soundsStore.getTotalSoundCount()
	);
	const recordedCount = $derived.by(() => {
		if (!settingsStore.singleRecordingMode) return recorded.length;
		const allFiles = soundsStore.flattenAllFiles();
		const eventFiles = new Map<string, string[]>();
		let standaloneCount = 0;
		for (const f of allFiles) {
			if (!f.soundEvent) {
				if (recorded.includes(f.path)) standaloneCount++;
			} else {
				const arr = eventFiles.get(f.soundEvent) ?? [];
				arr.push(f.path);
				eventFiles.set(f.soundEvent, arr);
			}
		}
		let eventCount = 0;
		for (const paths of eventFiles.values()) {
			if (paths.some((p) => recorded.includes(p))) eventCount++;
		}
		return standaloneCount + eventCount;
	});
	const selectedCount = $derived.by(() => {
		const size = soundsStore.selectedPaths.size;
		if (!settingsStore.singleRecordingMode || size === 0) return size;
		const allFiles = soundsStore.flattenAllFiles();
		const selected = allFiles.filter((f) => soundsStore.selectedPaths.has(f.path));
		const seenEvents = new Set<string>();
		let count = 0;
		for (const f of selected) {
			if (!f.soundEvent) {
				count++;
			} else if (!seenEvents.has(f.soundEvent)) {
				seenEvents.add(f.soundEvent);
				count++;
			}
		}
		return count;
	});

	const displayNodes = $derived.by(() => {
		const base = soundsStore.searchQuery ? soundsStore.searchResults : soundsStore.currentNodes;
		if (settingsStore.singleRecordingMode) {
			return deduplicateForSingleMode(base);
		}
		return base;
	});

	function handleTileClick(node: SoundNode) {
		if (node.nodeType === 'directory') {
			if (soundsStore.searchQuery) {
				soundsStore.toggleSelect(node);
			} else {
				soundsStore.enterDirectory(node.name);
			}
		} else {
			soundsStore.toggleSelect(node);
		}
	}

	function handleTileCheck(node: SoundNode, _checked: boolean) {
		soundsStore.toggleSelect(node);
	}

	function handleDragSelect(nodes: SoundNode[]) {
		soundsStore.selectNodes(nodes);
	}

	function handleRecordSelected() {
		const selected = soundsStore.getSelectedSounds();
		if (selected.length === 0) return;

		const alreadyRecorded = selected.filter((s) => recorded.includes(s.path));
		if (alreadyRecorded.length > 0) {
			showRerecordWarning = true;
		} else {
			startRecording(selected);
		}
	}

	function deduplicateForSingleMode(sounds: SoundNode[]): SoundNode[] {
		const seenEvents = new Set<string>();
		return sounds.filter((s) => {
			if (!s.soundEvent) return true;
			if (seenEvents.has(s.soundEvent)) return false;
			seenEvents.add(s.soundEvent);
			return true;
		});
	}

	function startRecording(sounds: SoundNode[]) {
		if (!packStore.currentPack) return;
		const isSingleMode = settingsStore.singleRecordingMode;
		const finalSounds = isSingleMode ? deduplicateForSingleMode(sounds) : sounds;
		recordingStore.setup(finalSounds, packId, packStore.currentPack.versionId, isSingleMode);
		goto('/record');
	}

	function handleRerecord() {
		showRerecordWarning = false;
		startRecording(soundsStore.getSelectedSounds());
	}

	function handleSkipRecorded() {
		showRerecordWarning = false;
		const selected = soundsStore.getSelectedSounds();
		const unrecorded = selected.filter((s) => !recorded.includes(s.path));
		if (unrecorded.length > 0) {
			startRecording(unrecorded);
		}
	}
</script>

<div class="flex flex-col h-full">
	<!-- Pack header -->
	<div class="px-4 py-3 border-b border-[var(--border-color)] bg-[var(--bg-secondary)]">
		<div class="flex items-center gap-3">
			<button
				onclick={() => goto('/')}
				class="text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors shrink-0"
				aria-label="Back to home"
			>
				<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
					<polyline points="9 22 9 12 15 12 15 22"/>
				</svg>
			</button>
			<button
				onclick={() => soundsStore.navigateTo([])}
				class="text-left shrink-0 hover:opacity-80 transition-opacity"
			>
				<h1 class="text-lg font-bold leading-tight">{packStore.currentPack?.name ?? 'Loading...'}</h1>
				<p class="text-xs text-[var(--text-muted)]">
					{recordedCount} / {totalSounds} sounds recorded
					{#if packStore.currentPack}
						&middot; {packStore.currentPack.versionId}
					{/if}
				</p>
			</button>
			<div class="flex-1 mx-3">
				<SearchBar
					value={soundsStore.searchQuery}
					oninput={(v) => soundsStore.setSearch(v)}
				/>
			</div>
			<div class="flex items-center gap-3 shrink-0">
				{#if selectedCount > 0}
					<div class="text-right">
						<div class="text-sm text-[var(--text-secondary)]">
							{selectedCount} items selected
						</div>
						<button
							onclick={() => soundsStore.clearSelection()}
							class="text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
						>
							Clear selection
						</button>
					</div>
				{/if}
				<button
					onclick={handleRecordSelected}
					disabled={selectedCount === 0}
					class="px-4 py-2 rounded-lg bg-cta text-white hover:bg-cta-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium"
				>
					Record
				</button>
			</div>
		</div>

		{#if !soundsStore.searchQuery && soundsStore.currentPath.length > 0}
			<div class="mt-1">
				<Breadcrumb
					crumbs={soundsStore.breadcrumbs}
					onnavigate={(path) => soundsStore.navigateTo(path)}
				/>
			</div>
		{/if}
	</div>

	<!-- Tile grid -->
	<div class="flex-1 overflow-auto">
		{#if soundsStore.loading}
			<div class="flex items-center justify-center py-12 text-[var(--text-muted)]">
				Loading sounds...
			</div>
		{:else}
			<TileGrid
				nodes={displayNodes}
				recordedSounds={recorded}
				selectedPaths={soundsStore.selectedPaths}
				showPath={!!soundsStore.searchQuery}
				showBackTile={!soundsStore.searchQuery && soundsStore.currentPath.length > 0}
				showClearSearch={!!soundsStore.searchQuery}
				ontileclick={handleTileClick}
				ontilecheck={handleTileCheck}
				ondragselect={handleDragSelect}
				onbackclick={() => soundsStore.navigateTo(soundsStore.currentPath.slice(0, -1))}
				onclearsearch={() => soundsStore.setSearch('')}
			/>
		{/if}
	</div>
</div>

<WarningDialog
	open={showRerecordWarning}
	title="Re-record sounds?"
	message="Some of the selected sounds already have recordings. Would you like to re-record all of them, or skip the ones already recorded?"
	confirmText="Skip recorded"
	dangerousConfirmText="Re-record all"
	cancelText="Cancel"
	onconfirm={handleSkipRecorded}
	ondangerous={handleRerecord}
	oncancel={() => (showRerecordWarning = false)}
/>
