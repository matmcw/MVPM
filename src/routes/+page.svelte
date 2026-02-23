<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { packStore } from '$lib/stores/pack.svelte';
	import PackCard from '$lib/components/PackCard.svelte';
	import type { PackMeta } from '$lib/utils/api';

	let duplicateTarget = $state<PackMeta | null>(null);
	let duplicateName = $state('');
	let duplicateError = $state('');

	let deleteTarget = $state<PackMeta | null>(null);
	let deleteConfirmText = $state('');
	let deleteError = $state('');
	let deleting = $state(false);

	onMount(async () => {
		await packStore.loadPacks();
	});

	function startDelete(pack: PackMeta) {
		deleteTarget = pack;
		deleteConfirmText = '';
		deleteError = '';
	}

	async function confirmDelete() {
		if (!deleteTarget) return;
		if (deleteConfirmText !== deleteTarget.name) {
			deleteError = 'Pack name does not match.';
			return;
		}
		deleting = true;
		try {
			await packStore.deletePack(deleteTarget.id);
			deleteTarget = null;
		} catch (e) {
			deleteError = e instanceof Error ? e.message : String(e);
		} finally {
			deleting = false;
		}
	}

	function startDuplicate(pack: PackMeta) {
		duplicateTarget = pack;
		duplicateName = `${pack.name} (Copy)`;
		duplicateError = '';
	}

	async function confirmDuplicate() {
		if (!duplicateTarget) return;
		if (!duplicateName.trim()) {
			duplicateError = 'Name is required.';
			return;
		}
		try {
			await packStore.duplicatePack(duplicateTarget.id, duplicateName.trim());
			duplicateTarget = null;
		} catch (e) {
			duplicateError = e instanceof Error ? e.message : String(e);
		}
	}
</script>

<div class="max-w-3xl mx-auto p-6">
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-2xl font-bold">Your Packs</h1>
		<button
			onclick={() => goto('/create/version')}
			class="px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors font-medium"
		>
			+ Create New Pack
		</button>
	</div>

	{#if packStore.loading}
		<div class="flex items-center justify-center py-12 text-[var(--text-muted)]">
			Loading packs...
		</div>
	{:else if packStore.packs.length === 0}
		<div class="flex flex-col items-center justify-center py-16 text-center">
			<svg class="mb-4 text-[var(--text-muted)]" xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
				<path d="M12 3v18"/>
				<path d="M3 12h18"/>
			</svg>
			<h2 class="text-lg font-medium text-[var(--text-secondary)] mb-2">No packs yet</h2>
			<p class="text-[var(--text-muted)] mb-4">Create your first voice pack to get started.</p>
			<button
				onclick={() => goto('/create/version')}
				class="px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors"
			>
				Create New Pack
			</button>
		</div>
	{:else}
		<div class="space-y-3">
			{#each packStore.packs as pack (pack.id)}
				<PackCard {pack} onduplicate={startDuplicate} ondelete={startDelete} />
			{/each}
		</div>
	{/if}

	<div class="mt-8 pt-6 border-t border-[var(--border-color)]">
		<button
			onclick={() => goto('/help')}
			class="text-sm text-[var(--text-muted)] hover:text-[var(--text-secondary)] transition-colors"
		>
			Need help getting started? View the guide
		</button>
	</div>
</div>

<!-- Duplicate dialog -->
{#if duplicateTarget}
	<div class="fixed inset-0 z-50 flex items-center justify-center">
		<div class="absolute inset-0 bg-black/50" onclick={() => (duplicateTarget = null)} role="presentation"></div>
		<div class="relative bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl shadow-lg p-6 max-w-md w-full mx-4">
			<h3 class="text-lg font-semibold mb-4">Duplicate Pack</h3>
			<p class="text-sm text-[var(--text-secondary)] mb-3">
				Enter a name for the duplicate of "{duplicateTarget.name}":
			</p>
			<input
				type="text"
				bind:value={duplicateName}
				class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] focus:outline-none focus:border-primary"
				onkeydown={(e) => e.key === 'Enter' && confirmDuplicate()}
			/>
			{#if duplicateError}
				<p class="text-sm text-danger mt-2">{duplicateError}</p>
			{/if}
			<div class="flex gap-3 justify-end mt-4">
				<button
					onclick={() => (duplicateTarget = null)}
					class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors"
				>
					Cancel
				</button>
				<button
					onclick={confirmDuplicate}
					class="px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors"
				>
					Duplicate
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete confirmation dialog -->
{#if deleteTarget}
	<div class="fixed inset-0 z-50 flex items-center justify-center">
		<div class="absolute inset-0 bg-black/50" onclick={() => (deleteTarget = null)} role="presentation"></div>
		<div class="relative bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl shadow-lg p-6 max-w-md w-full mx-4">
			<h3 class="text-lg font-semibold mb-2">Delete Pack</h3>
			<p class="text-[var(--text-secondary)] mb-4">
				This will permanently delete <strong>"{deleteTarget.name}"</strong> and all its recordings. This cannot be undone.
			</p>
			<p class="text-sm text-[var(--text-secondary)] mb-2">
				Type the pack name to confirm:
			</p>
			<input
				type="text"
				bind:value={deleteConfirmText}
				class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] focus:outline-none focus:border-danger"
				placeholder={deleteTarget.name}
				onkeydown={(e) => e.key === 'Enter' && confirmDelete()}
			/>
			{#if deleteError}
				<p class="text-sm text-danger mt-2">{deleteError}</p>
			{/if}
			<div class="flex gap-3 justify-end mt-4">
				<button
					onclick={() => (deleteTarget = null)}
					class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors"
				>
					Cancel
				</button>
				<button
					onclick={confirmDelete}
					disabled={deleting || deleteConfirmText !== deleteTarget.name}
					class="px-4 py-2 rounded-lg bg-danger text-white hover:bg-danger-hover transition-colors disabled:opacity-50"
				>
					{deleting ? 'Deleting...' : 'Delete Pack'}
				</button>
			</div>
		</div>
	</div>
{/if}
