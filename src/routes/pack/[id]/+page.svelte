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
	const totalSounds = $derived(soundsStore.getTotalSoundCount());
	const recordedCount = $derived(recorded.length);
	const selectedCount = $derived(soundsStore.selectedPaths.size);

	const displayNodes = $derived(
		soundsStore.searchQuery ? soundsStore.searchResults : soundsStore.currentNodes
	);

	function handleTileClick(node: SoundNode) {
		if (soundsStore.searchQuery && node.nodeType === 'file') {
			soundsStore.navigateToFile(node);
			return;
		}
		soundsStore.toggleSelect(node);
	}

	function handleTileDblClick(node: SoundNode) {
		if (node.nodeType === 'directory') {
			soundsStore.enterDirectory(node.name);
		}
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

	function startRecording(sounds: SoundNode[]) {
		if (!packStore.currentPack) return;
		recordingStore.setup(sounds, packId, packStore.currentPack.versionId);
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
		<div class="flex items-center justify-between mb-2">
			<div class="flex items-center gap-3">
				<button
					onclick={() => goto('/')}
					class="text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
				>
					<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<line x1="19" y1="12" x2="5" y2="12"/>
						<polyline points="12 19 5 12 12 5"/>
					</svg>
				</button>
				<div>
					<h1 class="text-lg font-bold">{packStore.currentPack?.name ?? 'Loading...'}</h1>
					<p class="text-xs text-[var(--text-muted)]">
						{recordedCount} / {totalSounds} sounds recorded
						{#if packStore.currentPack}
							&middot; {packStore.currentPack.versionId}
						{/if}
					</p>
				</div>
			</div>
			<div class="flex items-center gap-2">
				{#if selectedCount > 0}
					<span class="text-sm text-[var(--text-secondary)]">
						{selectedCount} selected
					</span>
					<button
						onclick={() => soundsStore.clearSelection()}
						class="text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
					>
						Clear
					</button>
				{/if}
				<button
					onclick={handleRecordSelected}
					disabled={selectedCount === 0}
					class="px-4 py-2 rounded-lg bg-recording text-white hover:bg-danger-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium"
				>
					Record Selected ({selectedCount})
				</button>
			</div>
		</div>

		<div class="flex items-center gap-4">
			{#if !soundsStore.searchQuery}
				<Breadcrumb
					crumbs={soundsStore.breadcrumbs}
					onnavigate={(path) => soundsStore.navigateTo(path)}
				/>
			{/if}
			<div class="flex-1 max-w-xs ml-auto">
				<SearchBar
					value={soundsStore.searchQuery}
					oninput={(v) => soundsStore.setSearch(v)}
				/>
			</div>
		</div>
	</div>

	<!-- Tile grid -->
	<div class="flex-1 overflow-auto p-4">
		{#if soundsStore.loading}
			<div class="flex items-center justify-center py-12 text-[var(--text-muted)]">
				Loading sounds...
			</div>
		{:else}
			<TileGrid
				nodes={displayNodes}
				recordedSounds={recorded}
				selectedPaths={soundsStore.selectedPaths}
				ontileclick={handleTileClick}
				ontiledblclick={handleTileDblClick}
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
