<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { versionsStore } from '$lib/stores/versions.svelte';
	import DownloadProgress from '$lib/components/DownloadProgress.svelte';

	let releasesOnly = $state(true);
	let selectedVersion = $state<string | null>(null);
	let showDownload = $state(false);
	let searchFilter = $state('');

	onMount(async () => {
		await versionsStore.fetchVersions();
	});

	function filteredVersions() {
		let list = releasesOnly ? versionsStore.releases() : versionsStore.versions;
		if (searchFilter.trim()) {
			const q = searchFilter.toLowerCase();
			list = list.filter((v) => v.id.toLowerCase().includes(q));
		}
		return list;
	}

	async function handleNext() {
		if (!selectedVersion) return;
		const downloaded = await versionsStore.isDownloaded(selectedVersion);
		if (downloaded) {
			goto(`/create/details?version=${selectedVersion}`);
		} else {
			showDownload = true;
		}
	}

	function onDownloadSuccess() {
		showDownload = false;
		if (selectedVersion) {
			goto(`/create/details?version=${selectedVersion}`);
		}
	}
</script>

<div class="max-w-2xl mx-auto p-6">
	<div class="mb-6">
		<h1 class="text-2xl font-bold mb-1">Create New Pack</h1>
		<p class="text-[var(--text-secondary)]">Step 1 of 3: Select Minecraft version</p>
	</div>

	<div class="flex items-center gap-4 mb-4">
		<input
			type="text"
			placeholder="Filter versions..."
			bind:value={searchFilter}
			class="flex-1 px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] focus:outline-none focus:border-primary"
		/>
		<label class="flex items-center gap-2 text-sm text-[var(--text-secondary)] whitespace-nowrap cursor-pointer">
			<input type="checkbox" bind:checked={releasesOnly} class="rounded" />
			Releases only
		</label>
	</div>

	{#if versionsStore.loading}
		<div class="flex items-center justify-center py-12 text-[var(--text-muted)]">
			Loading versions...
		</div>
	{:else if versionsStore.error}
		<div class="text-center py-8">
			<p class="text-danger mb-4">{versionsStore.error}</p>
			<button
				onclick={() => versionsStore.fetchVersions()}
				class="px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors"
			>
				Retry
			</button>
		</div>
	{:else}
		<div class="border border-[var(--border-color)] rounded-lg max-h-[400px] overflow-y-auto">
			{#each filteredVersions() as version (version.id)}
				<button
					onclick={() => (selectedVersion = version.id)}
					class="w-full flex items-center justify-between px-4 py-3 border-b border-[var(--border-color)] last:border-b-0 transition-colors text-left
						{selectedVersion === version.id
							? 'bg-selected-light text-primary'
							: 'hover:bg-[var(--bg-secondary)]'}"
				>
					<span class="font-medium">{version.id}</span>
					<span class="text-xs text-[var(--text-muted)] capitalize">{version.versionType}</span>
				</button>
			{/each}
		</div>
	{/if}

	<div class="flex items-center justify-between mt-6">
		<button
			onclick={() => goto('/')}
			class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors"
		>
			Cancel
		</button>
		<button
			onclick={handleNext}
			disabled={!selectedVersion}
			class="px-6 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
		>
			Next
		</button>
	</div>
</div>

{#if showDownload && selectedVersion}
	<DownloadProgress
		open={showDownload}
		versionId={selectedVersion}
		onclose={() => (showDownload = false)}
		onsuccess={onDownloadSuccess}
	/>
{/if}
