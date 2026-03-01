<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { packStore } from '$lib/stores/pack.svelte';
	import { versionsStore } from '$lib/stores/versions.svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import WarningDialog from '$lib/components/WarningDialog.svelte';
	import DownloadProgress from '$lib/components/DownloadProgress.svelte';

	const packId = $derived(page.params.id);

	let name = $state('');
	let description = $state('');
	let error = $state('');
	let saving = $state(false);

	// Version change
	let showVersionChange = $state(false);
	let newVersionId = $state('');
	let showVersionWarning = $state(false);
	let showDownload = $state(false);

	// Duplicate
	let showDuplicate = $state(false);
	let duplicateName = $state('');
	let duplicateError = $state('');

	// Delete
	let showDelete = $state(false);
	let deleteConfirmText = $state('');
	let deleteError = $state('');
	let deleting = $state(false);

	onMount(async () => {
		await packStore.loadPack(packId);
		if (packStore.currentPack) {
			name = packStore.currentPack.name;
			description = packStore.currentPack.description;
		}
		await versionsStore.fetchVersions();
	});

	async function saveChanges() {
		if (!name.trim() || !description.trim()) return;
		saving = true;
		error = '';
		try {
			await packStore.updatePack(packId, name.trim(), description.trim());
			goto(`/pack/${packId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	async function changeIcon() {
		const result = await open({
			multiple: false,
			directory: false,
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg'] }],
		});
		if (result) {
			try {
				await packStore.updatePack(packId, undefined, undefined, result as string);
			} catch (e) {
				error = e instanceof Error ? e.message : String(e);
			}
		}
	}

	async function handleVersionChange() {
		if (!newVersionId || !packStore.currentPack) return;
		if (newVersionId === packStore.currentPack.versionId) return;

		const downloaded = await versionsStore.isDownloaded(newVersionId);
		if (!downloaded) {
			showDownload = true;
		} else {
			showVersionWarning = true;
		}
	}

	function onDownloadSuccess() {
		showDownload = false;
		showVersionWarning = true;
	}

	async function confirmVersionChange() {
		showVersionWarning = false;
		try {
			await packStore.changeVersion(packId, newVersionId);
			goto(`/pack/${packId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function confirmDelete() {
		if (!packStore.currentPack) return;
		if (deleteConfirmText !== packStore.currentPack.name) {
			deleteError = 'Pack name does not match.';
			return;
		}
		deleting = true;
		try {
			await packStore.deletePack(packId);
			goto('/');
		} catch (e) {
			deleteError = e instanceof Error ? e.message : String(e);
		} finally {
			deleting = false;
		}
	}

	async function handleDuplicate() {
		if (!duplicateName.trim()) {
			duplicateError = 'Name is required.';
			return;
		}
		try {
			const dup = await packStore.duplicatePack(packId, duplicateName.trim());
			showDuplicate = false;
			goto(`/pack/${dup.id}`);
		} catch (e) {
			duplicateError = e instanceof Error ? e.message : String(e);
		}
	}
</script>

<div class="max-w-2xl mx-auto p-6">
	<div class="flex items-center gap-3 mb-6">
		<button
			onclick={() => goto(`/pack/${packId}`)}
			class="text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<line x1="19" y1="12" x2="5" y2="12"/>
				<polyline points="12 19 5 12 12 5"/>
			</svg>
		</button>
		<h1 class="text-2xl font-bold">Edit Pack</h1>
	</div>

	{#if error}
		<p class="text-danger text-sm mb-4">{error}</p>
	{/if}

	<div class="space-y-6">
		<!-- Name & Description -->
		<div class="space-y-4">
			<div>
				<label for="edit-name" class="block text-sm font-medium mb-1">Pack Name</label>
				<input
					id="edit-name"
					type="text"
					bind:value={name}
					class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] focus:outline-none focus:border-primary"
				/>
			</div>
			<div>
				<label for="edit-desc" class="block text-sm font-medium mb-1">Description</label>
				<textarea
					id="edit-desc"
					bind:value={description}
					rows={3}
					class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] focus:outline-none focus:border-primary resize-none"
				></textarea>
			</div>
			<div class="flex gap-3">
				<button
					onclick={saveChanges}
					disabled={saving || !name.trim() || !description.trim()}
					class="px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors disabled:opacity-50"
				>
					{saving ? 'Saving...' : 'Save Changes'}
				</button>
			</div>
		</div>

		<!-- Icon -->
		<div class="py-4 border-t border-[var(--border-color)]">
			<h3 class="font-medium mb-2">Pack Icon</h3>
			<div class="flex items-center gap-3">
				{#if packStore.currentPack?.iconPath}
					<img
						src={convertFileSrc(packStore.currentPack.iconPath)}
						alt="Pack icon"
						class="w-12 h-12 rounded-lg object-cover border border-[var(--border-color)]"
					/>
				{/if}
				<button
					onclick={changeIcon}
					class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors text-sm"
				>
					{packStore.currentPack?.hasIcon ? 'Change Icon' : 'Add Icon'}
				</button>
			</div>
		</div>

		<!-- Version Change -->
		<div class="py-4 border-t border-[var(--border-color)]">
			<h3 class="font-medium mb-2">Minecraft Version</h3>
			<p class="text-sm text-[var(--text-muted)] mb-3">
				Current: {packStore.currentPack?.versionId ?? '...'}
			</p>
			<div class="flex gap-3 items-center">
				<select
					bind:value={newVersionId}
					class="px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] flex-1"
				>
					<option value="">Select new version...</option>
					{#each versionsStore.releases() as version (version.id)}
						<option value={version.id}>{version.id}</option>
					{/each}
				</select>
				<button
					onclick={handleVersionChange}
					disabled={!newVersionId || newVersionId === packStore.currentPack?.versionId}
					class="px-4 py-2 rounded-lg bg-warning text-white hover:opacity-90 transition-colors disabled:opacity-50"
				>
					Change Version
				</button>
			</div>
		</div>

		<!-- Duplicate -->
		<div class="py-4 border-t border-[var(--border-color)]">
			<h3 class="font-medium mb-2">Duplicate Pack</h3>
			<p class="text-sm text-[var(--text-muted)] mb-3">
				Create a copy of this pack with all its recordings.
			</p>
			<button
				onclick={() => {
					duplicateName = `${packStore.currentPack?.name ?? 'Pack'} (Copy)`;
					duplicateError = '';
					showDuplicate = true;
				}}
				class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors text-sm"
			>
				Duplicate Pack
			</button>
		</div>

		<!-- Delete -->
		<div class="py-4 border-t border-[var(--border-color)]">
			<h3 class="font-medium mb-2 text-danger">Delete Pack</h3>
			<p class="text-sm text-[var(--text-muted)] mb-3">
				Permanently delete this pack and all its recordings. This cannot be undone.
			</p>
			<button
				onclick={() => {
					deleteConfirmText = '';
					deleteError = '';
					showDelete = true;
				}}
				class="px-4 py-2 rounded-lg border border-danger/50 text-danger hover:bg-danger/10 transition-colors text-sm"
			>
				Delete Pack
			</button>
		</div>
	</div>
</div>

<!-- Version change warning -->
<WarningDialog
	open={showVersionWarning}
	title="Change Version?"
	message="Changing version will remove sounds that don't exist in the new version and add new sound slots. Recordings for sounds that exist in both versions will be kept. We recommend duplicating the pack first."
	confirmText="Change Version"
	cancelText="Cancel"
	onconfirm={confirmVersionChange}
	oncancel={() => (showVersionWarning = false)}
/>

<!-- Download modal -->
{#if showDownload && newVersionId}
	<DownloadProgress
		open={showDownload}
		versionId={newVersionId}
		onclose={() => (showDownload = false)}
		onsuccess={onDownloadSuccess}
	/>
{/if}

<!-- Delete confirmation dialog -->
{#if showDelete}
	<div class="fixed inset-0 z-50 flex items-center justify-center">
		<div class="absolute inset-0 bg-black/50" onclick={() => (showDelete = false)} role="presentation"></div>
		<div class="relative bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl shadow-lg p-6 max-w-md w-full mx-4">
			<h3 class="text-lg font-semibold mb-2">Delete Pack</h3>
			<p class="text-[var(--text-secondary)] mb-4">
				This will permanently delete <strong>"{packStore.currentPack?.name}"</strong> and all its recordings. This cannot be undone.
			</p>
			<p class="text-sm text-[var(--text-secondary)] mb-2">
				Type the pack name to confirm:
			</p>
			<input
				type="text"
				bind:value={deleteConfirmText}
				class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] focus:outline-none focus:border-danger"
				placeholder={packStore.currentPack?.name}
				onkeydown={(e) => e.key === 'Enter' && confirmDelete()}
			/>
			{#if deleteError}
				<p class="text-sm text-danger mt-2">{deleteError}</p>
			{/if}
			<div class="flex gap-3 justify-end mt-4">
				<button
					onclick={() => (showDelete = false)}
					class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors"
				>
					Cancel
				</button>
				<button
					onclick={confirmDelete}
					disabled={deleting || deleteConfirmText !== packStore.currentPack?.name}
					class="px-4 py-2 rounded-lg bg-danger text-white hover:bg-danger-hover transition-colors disabled:opacity-50"
				>
					{deleting ? 'Deleting...' : 'Delete Pack'}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Duplicate dialog -->
{#if showDuplicate}
	<div class="fixed inset-0 z-50 flex items-center justify-center">
		<div class="absolute inset-0 bg-black/50" onclick={() => (showDuplicate = false)} role="presentation"></div>
		<div class="relative bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl shadow-lg p-6 max-w-md w-full mx-4">
			<h3 class="text-lg font-semibold mb-4">Duplicate Pack</h3>
			<input
				type="text"
				bind:value={duplicateName}
				class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] focus:outline-none focus:border-primary"
				placeholder="New pack name"
				onkeydown={(e) => e.key === 'Enter' && handleDuplicate()}
			/>
			{#if duplicateError}
				<p class="text-sm text-danger mt-2">{duplicateError}</p>
			{/if}
			<div class="flex gap-3 justify-end mt-4">
				<button
					onclick={() => (showDuplicate = false)}
					class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors"
				>
					Cancel
				</button>
				<button
					onclick={handleDuplicate}
					class="px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors"
				>
					Duplicate
				</button>
			</div>
		</div>
	</div>
{/if}
