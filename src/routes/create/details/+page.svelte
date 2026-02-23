<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	let name = $state('');
	let description = $state('');
	let nameError = $state('');

	const version = $derived(page.url.searchParams.get('version') ?? '');

	function handleNext() {
		if (!name.trim()) {
			nameError = 'Pack name is required.';
			return;
		}
		nameError = '';
		const params = new URLSearchParams({
			version,
			name: name.trim(),
			description: description.trim(),
		});
		goto(`/create/icon?${params.toString()}`);
	}
</script>

<div class="max-w-2xl mx-auto p-6">
	<div class="mb-6">
		<h1 class="text-2xl font-bold mb-1">Create New Pack</h1>
		<p class="text-[var(--text-secondary)]">Step 2 of 3: Pack details (version: {version})</p>
	</div>

	<div class="space-y-4">
		<div>
			<label for="pack-name" class="block text-sm font-medium mb-1">Pack Name *</label>
			<input
				id="pack-name"
				type="text"
				bind:value={name}
				placeholder="My Voice Pack"
				class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] focus:outline-none focus:border-primary"
				onkeydown={(e) => e.key === 'Enter' && handleNext()}
			/>
			{#if nameError}
				<p class="text-sm text-danger mt-1">{nameError}</p>
			{/if}
		</div>

		<div>
			<label for="pack-desc" class="block text-sm font-medium mb-1">Description *</label>
			<textarea
				id="pack-desc"
				bind:value={description}
				placeholder="A custom voice pack for Minecraft"
				rows={3}
				class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] focus:outline-none focus:border-primary resize-none"
			></textarea>
		</div>
	</div>

	<div class="flex items-center justify-between mt-6">
		<button
			onclick={() => goto('/create/version')}
			class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors"
		>
			Back
		</button>
		<button
			onclick={handleNext}
			disabled={!name.trim() || !description.trim()}
			class="px-6 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
		>
			Next
		</button>
	</div>
</div>
