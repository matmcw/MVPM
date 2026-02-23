<script lang="ts">
	import type { SoundNode } from '$lib/utils/api';
	import Tile from './Tile.svelte';
	import { isDirectoryComplete } from '$lib/stores/sounds.svelte';

	interface Props {
		nodes: SoundNode[];
		recordedSounds: string[];
		selectedPaths: Set<string>;
		ontileclick: (node: SoundNode) => void;
		ontiledblclick: (node: SoundNode) => void;
	}

	let { nodes, recordedSounds, selectedPaths, ontileclick, ontiledblclick }: Props = $props();

	function isTileRecorded(node: SoundNode): boolean {
		if (node.nodeType === 'file') {
			return recordedSounds.includes(node.path);
		}
		return isDirectoryComplete(node, recordedSounds);
	}

	function isTileSelected(node: SoundNode): boolean {
		if (node.nodeType === 'file') {
			return selectedPaths.has(node.path);
		}
		// Directory is selected if all its files are selected
		return false;
	}
</script>

{#if nodes.length === 0}
	<div class="flex items-center justify-center py-12 text-[var(--text-muted)]">
		No sounds in this directory
	</div>
{:else}
	<div class="grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-2">
		{#each nodes as node (node.path)}
			<Tile
				{node}
				isRecorded={isTileRecorded(node)}
				isSelected={isTileSelected(node)}
				onclick={() => ontileclick(node)}
				ondblclick={() => ontiledblclick(node)}
			/>
		{/each}
	</div>
{/if}
