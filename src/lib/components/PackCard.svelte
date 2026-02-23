<script lang="ts">
	import type { PackMeta } from '$lib/utils/api';
	import { goto } from '$app/navigation';

	interface Props {
		pack: PackMeta;
		onduplicate: (pack: PackMeta) => void;
		ondelete: (pack: PackMeta) => void;
	}

	let { pack, onduplicate, ondelete }: Props = $props();

	const recordedCount = $derived(pack.recordedSounds.length);
</script>

<div class="flex items-center gap-4 p-4 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] hover:border-[var(--border-hover)] transition-colors group">
	<!-- Click area for opening pack -->
	<button
		onclick={() => goto(`/pack/${pack.id}`)}
		class="flex-1 text-left min-w-0"
	>
		<h3 class="font-semibold text-[var(--text-primary)] truncate">{pack.name}</h3>
		<p class="text-sm text-[var(--text-secondary)] truncate">{pack.description}</p>
		<div class="flex items-center gap-3 mt-2">
			<span class="text-xs text-[var(--text-muted)] bg-[var(--bg-tertiary)] px-2 py-0.5 rounded">
				{pack.versionId}
			</span>
			<span class="text-xs text-[var(--text-muted)]">
				{recordedCount} sound{recordedCount !== 1 ? 's' : ''} recorded
			</span>
		</div>
	</button>

	<!-- Actions -->
	<div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
		<button
			onclick={() => goto(`/pack/${pack.id}/edit`)}
			class="p-2 rounded-lg hover:bg-[var(--bg-tertiary)] transition-colors"
			title="Edit pack"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
				<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
			</svg>
		</button>
		<button
			onclick={() => onduplicate(pack)}
			class="p-2 rounded-lg hover:bg-[var(--bg-tertiary)] transition-colors"
			title="Duplicate pack"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
				<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
			</svg>
		</button>
		<button
			onclick={() => ondelete(pack)}
			class="p-2 rounded-lg hover:bg-danger/10 text-[var(--text-muted)] hover:text-danger transition-colors"
			title="Delete pack"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<polyline points="3 6 5 6 21 6"/>
				<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
				<line x1="10" y1="11" x2="10" y2="17"/>
				<line x1="14" y1="11" x2="14" y2="17"/>
			</svg>
		</button>
	</div>
</div>
