<script lang="ts">
	import { goto } from '$app/navigation';
	import { packStore } from '$lib/stores/pack.svelte';
	import { createWizardStore } from '$lib/stores/createWizard.svelte';
	import { open } from '@tauri-apps/plugin-dialog';

	let iconPath = $state<string | null>(createWizardStore.iconPath);
	let creating = $state(false);
	let error = $state('');

	const version = $derived(createWizardStore.version ?? '');
	const name = $derived(createWizardStore.name);
	const description = $derived(createWizardStore.description);

	async function selectIcon() {
		const result = await open({
			multiple: false,
			directory: false,
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg'] }],
		});
		if (result) {
			iconPath = result as string;
			createWizardStore.iconPath = result as string;
		}
	}

	async function createPack(skipIcon = false) {
		creating = true;
		error = '';
		try {
			const pack = await packStore.createPack(
				name,
				description,
				version,
				skipIcon ? undefined : iconPath ?? undefined,
			);
			createWizardStore.clear();
			goto(`/pack/${pack.id}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			creating = false;
		}
	}
</script>

<div class="max-w-2xl mx-auto p-6 min-h-full flex flex-col justify-center">
	<div class="mb-6">
		<h1 class="text-2xl font-bold mb-1">Create New Pack</h1>
		<p class="text-[var(--text-secondary)]">Step 3 of 3: Pack icon (optional)</p>
	</div>

	<div class="flex flex-col items-center gap-4 py-8">
		{#if iconPath}
			<div class="w-32 h-32 rounded-lg border-2 border-dashed border-primary bg-[var(--bg-secondary)] flex items-center justify-center overflow-hidden">
				<span class="text-sm text-[var(--text-secondary)] text-center px-2 break-all">
					{iconPath.split(/[/\\]/).pop()}
				</span>
			</div>
			<button
				onclick={selectIcon}
				class="text-sm text-primary hover:underline"
			>
				Change icon
			</button>
		{:else}
			<button
				onclick={selectIcon}
				class="w-32 h-32 rounded-lg border-2 border-dashed border-[var(--border-color)] hover:border-primary bg-[var(--bg-secondary)] flex flex-col items-center justify-center gap-2 transition-colors"
			>
				<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="text-[var(--text-muted)]">
					<rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
					<circle cx="8.5" cy="8.5" r="1.5"/>
					<polyline points="21 15 16 10 5 21"/>
				</svg>
				<span class="text-xs text-[var(--text-muted)]">Choose icon</span>
			</button>
		{/if}

		<p class="text-sm text-[var(--text-muted)]">
			PNG recommended, 64x64 or larger. This is optional.
		</p>
	</div>

	{#if error}
		<p class="text-danger text-sm mb-4 text-center">{error}</p>
	{/if}

	<div class="flex items-center justify-between">
		<button
			onclick={() => goto('/create/details')}
			class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors"
		>
			Back
		</button>
		<div class="flex gap-3">
			<button
				onclick={() => createPack(true)}
				disabled={creating}
				class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors disabled:opacity-50"
			>
				Skip
			</button>
			<button
				onclick={() => createPack(false)}
				disabled={creating || !iconPath}
				class="px-6 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{creating ? 'Creating...' : 'Create Pack'}
			</button>
		</div>
	</div>
</div>
