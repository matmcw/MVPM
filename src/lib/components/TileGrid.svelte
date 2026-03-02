<script lang="ts">
	import type { SoundNode } from '$lib/utils/api';
	import Tile from './Tile.svelte';
	import { isDirectoryComplete } from '$lib/stores/sounds.svelte';

	interface Props {
		nodes: SoundNode[];
		recordedSounds: string[];
		selectedPaths: Set<string>;
		showPath?: boolean;
		showBackTile?: boolean;
		showClearSearch?: boolean;
		ontileclick: (node: SoundNode) => void;
		ontilecheck: (node: SoundNode, checked: boolean) => void;
		ondragselect: (nodes: SoundNode[]) => void;
		onbackclick?: () => void;
		onclearsearch?: () => void;
	}

	let { nodes, recordedSounds, selectedPaths, showPath, showBackTile, showClearSearch, ontileclick, ontilecheck, ondragselect, onbackclick, onclearsearch }: Props = $props();

	// Plain JS variables for drag tracking — NOT reactive to avoid triggering
	// Svelte re-renders during mousedown/mouseup/click event chains
	let _isDragging = false;
	let _dragPending = false;
	let _dragJustEnded = false; // suppresses post-drag click; cleared on next mousedown
	let _startX = 0;
	let _startY = 0;
	let _currentX = 0;
	let _currentY = 0;

	// Only this one is reactive — needed for rendering the selection box overlay
	let selectionBox = $state<{ left: number; top: number; width: number; height: number } | null>(null);
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

		// New mousedown = new user action → clear the post-drag flag.
		// This is what makes the flag impossible to get stuck: every
		// mousedown clears it, regardless of what element was clicked.
		_dragJustEnded = false;
		_dragPending = true;
		_startX = e.clientX;
		_startY = e.clientY;
		_currentX = e.clientX;
		_currentY = e.clientY;
	}

	function handleMouseMove(e: MouseEvent) {
		if (!_dragPending && !_isDragging) return;
		_currentX = e.clientX;
		_currentY = e.clientY;

		if (_dragPending && !_isDragging) {
			const dx = Math.abs(_currentX - _startX);
			const dy = Math.abs(_currentY - _startY);
			// 10px threshold — high enough that natural hand tremor during clicks
			// won't accidentally trigger drag mode (was 5px, caused click suppression)
			if (dx > 10 || dy > 10) {
				_isDragging = true;
				_dragPending = false;
			}
		}

		// Only update reactive state when actually dragging (for rendering the box)
		if (_isDragging) {
			selectionBox = {
				left: Math.min(_startX, _currentX),
				top: Math.min(_startY, _currentY),
				width: Math.abs(_currentX - _startX),
				height: Math.abs(_currentY - _startY),
			};
		}
	}

	function handleMouseUp() {
		_dragPending = false;
		if (!_isDragging) return;

		_isDragging = false;

		const selRect = {
			left: Math.min(_startX, _currentX),
			top: Math.min(_startY, _currentY),
			right: Math.max(_startX, _currentX),
			bottom: Math.max(_startY, _currentY),
		};

		// Clear the visual box
		selectionBox = null;

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

		// Flag to suppress the click that follows this mouseup. Cleared on
		// the next mousedown, so it can never get stuck for more than one click.
		_dragJustEnded = true;
	}
</script>

<svelte:window
	onmousemove={handleMouseMove}
	onmouseup={handleMouseUp}
	onblur={() => { _dragPending = false; _isDragging = false; _dragJustEnded = false; selectionBox = null; }}
/>

<div
	class="min-h-full p-4"
	style="user-select: none;"
	onmousedown={handleMouseDown}
	role="region"
>
	{#if nodes.length === 0 && !showClearSearch}
		<div class="flex items-center justify-center py-12 text-[var(--text-muted)]">
			No sounds in this directory
		</div>
	{:else}
		<div
			bind:this={gridRef}
			class="grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-2"
		>
			{#if showClearSearch && onclearsearch}
				<button
					onclick={() => { if (!_dragJustEnded) onclearsearch?.(); }}
					class="flex flex-col items-center justify-center p-3 rounded-lg border-2 border-[var(--border-color)] bg-[var(--bg-secondary)] hover:border-[var(--border-hover)] transition-all duration-150 min-h-[80px] cursor-pointer"
				>
					<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-[var(--text-muted)]">
						<line x1="18" y1="6" x2="6" y2="18"/>
						<line x1="6" y1="6" x2="18" y2="18"/>
					</svg>
					<span class="text-xs text-[var(--text-muted)] mt-1">Clear Search</span>
				</button>
			{:else if showBackTile && onbackclick}
				<button
					onclick={() => { if (!_dragJustEnded) onbackclick?.(); }}
					class="flex flex-col items-center justify-center p-3 rounded-lg border-2 border-[var(--border-color)] bg-[var(--bg-secondary)] hover:border-[var(--border-hover)] transition-all duration-150 min-h-[80px] cursor-pointer"
				>
					<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-[var(--text-muted)]">
						<line x1="19" y1="12" x2="5" y2="12"/>
						<polyline points="12 19 5 12 12 5"/>
					</svg>
					<span class="text-xs text-[var(--text-muted)] mt-1">Back</span>
				</button>
			{/if}
			{#each nodes as node (node.path)}
				<Tile
					{node}
					isRecorded={isTileRecorded(node)}
					isSelected={isTileSelected(node)}
					isPartiallySelected={isTilePartiallySelected(node)}
					{showPath}
					soundCount={node.nodeType === 'directory' ? collectFilesFromNode(node).length : undefined}
					onclick={() => { if (!_dragJustEnded) ontileclick(node); }}
					oncheckchange={(checked) => ontilecheck(node, checked)}
				/>
			{/each}
		</div>
	{/if}
</div>

{#if selectionBox}
	<div
		class="fixed z-50 border-2 border-selected bg-selected/10 pointer-events-none"
		style="
			left: {selectionBox.left}px;
			top: {selectionBox.top}px;
			width: {selectionBox.width}px;
			height: {selectionBox.height}px;
		"
	></div>
{/if}
