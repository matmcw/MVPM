<script lang="ts">
	import type { SoundNode } from '$lib/utils/api';
	import Tile from './Tile.svelte';
	import { isDirectoryComplete } from '$lib/stores/sounds.svelte';

	interface Props {
		nodes: SoundNode[];
		recordedSounds: string[];
		selectedPaths: Set<string>;
		ontileclick: (node: SoundNode) => void;
		ontilecheck: (node: SoundNode, checked: boolean) => void;
		ondragselect: (nodes: SoundNode[]) => void;
	}

	let { nodes, recordedSounds, selectedPaths, ontileclick, ontilecheck, ondragselect }: Props = $props();

	let isDragging = $state(false);
	let dragPending = $state(false);
	let dragStartX = $state(0);
	let dragStartY = $state(0);
	let dragCurrentX = $state(0);
	let dragCurrentY = $state(0);
	let gridRef = $state<HTMLElement | null>(null);

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
		if (!node.children || node.children.length === 0) return false;
		const allFiles = collectFilesFromNode(node);
		return allFiles.length > 0 && allFiles.every((f) => selectedPaths.has(f));
	}

	function isTilePartiallySelected(node: SoundNode): boolean {
		if (node.nodeType === 'file') return false;
		if (!node.children || node.children.length === 0) return false;
		const allFiles = collectFilesFromNode(node);
		const selectedCount = allFiles.filter((f) => selectedPaths.has(f)).length;
		return selectedCount > 0 && selectedCount < allFiles.length;
	}

	function collectFilesFromNode(node: SoundNode): string[] {
		const paths: string[] = [];
		if (node.nodeType === 'file') {
			paths.push(node.path);
		}
		if (node.children) {
			for (const child of node.children) {
				paths.push(...collectFilesFromNode(child));
			}
		}
		return paths;
	}

	function handleMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		if ((e.target as HTMLElement).closest('input[type="checkbox"]')) return;

		dragPending = true;
		dragStartX = e.clientX;
		dragStartY = e.clientY;
		dragCurrentX = e.clientX;
		dragCurrentY = e.clientY;
	}

	function handleMouseMove(e: MouseEvent) {
		if (!dragPending && !isDragging) return;
		dragCurrentX = e.clientX;
		dragCurrentY = e.clientY;

		if (dragPending && !isDragging) {
			const dx = Math.abs(dragCurrentX - dragStartX);
			const dy = Math.abs(dragCurrentY - dragStartY);
			if (dx > 5 || dy > 5) {
				isDragging = true;
				dragPending = false;
			}
		}
	}

	function handleMouseUp() {
		dragPending = false;
		if (!isDragging) return;
		isDragging = false;

		const selRect = {
			left: Math.min(dragStartX, dragCurrentX),
			top: Math.min(dragStartY, dragCurrentY),
			right: Math.max(dragStartX, dragCurrentX),
			bottom: Math.max(dragStartY, dragCurrentY),
		};

		if (!gridRef) return;
		const tileElements = gridRef.querySelectorAll('[data-path]');
		const nodesInRect: SoundNode[] = [];

		tileElements.forEach((el) => {
			const rect = el.getBoundingClientRect();
			if (
				rect.left < selRect.right &&
				rect.right > selRect.left &&
				rect.top < selRect.bottom &&
				rect.bottom > selRect.top
			) {
				const path = el.getAttribute('data-path');
				const node = nodes.find((n) => n.path === path);
				if (node) nodesInRect.push(node);
			}
		});

		if (nodesInRect.length > 0) {
			ondragselect(nodesInRect);
		}
	}
</script>

<svelte:window
	onmousemove={handleMouseMove}
	onmouseup={handleMouseUp}
	onblur={() => { dragPending = false; isDragging = false; }}
/>

{#if nodes.length === 0}
	<div class="flex items-center justify-center py-12 text-[var(--text-muted)]">
		No sounds in this directory
	</div>
{:else}
	<div
		bind:this={gridRef}
		class="grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-2 {isDragging ? 'no-select' : ''}"
		onmousedown={handleMouseDown}
		role="grid"
		tabindex="0"
	>
		{#each nodes as node (node.path)}
			<Tile
				{node}
				isRecorded={isTileRecorded(node)}
				isSelected={isTileSelected(node)}
				isPartiallySelected={isTilePartiallySelected(node)}
				onclick={() => ontileclick(node)}
				oncheckchange={(checked) => ontilecheck(node, checked)}
			/>
		{/each}
	</div>
{/if}

{#if isDragging}
	<div
		class="fixed z-50 border-2 border-selected bg-selected/10 pointer-events-none"
		style="
			left: {Math.min(dragStartX, dragCurrentX)}px;
			top: {Math.min(dragStartY, dragCurrentY)}px;
			width: {Math.abs(dragCurrentX - dragStartX)}px;
			height: {Math.abs(dragCurrentY - dragStartY)}px;
		"
	></div>
{/if}
