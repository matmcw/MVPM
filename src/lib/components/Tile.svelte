<script lang="ts">
	import type { SoundNode } from '$lib/utils/api';

	interface Props {
		node: SoundNode;
		isRecorded: boolean;
		isSelected: boolean;
		isPartiallySelected?: boolean;
		showPath?: boolean;
		soundCount?: number;
		onclick: () => void;
		oncheckchange: (checked: boolean) => void;
	}

	let { node, isRecorded, isSelected, isPartiallySelected, showPath, soundCount, onclick, oncheckchange }: Props = $props();

	const isDir = $derived(node.nodeType === 'directory');

	let checkboxRef = $state<HTMLInputElement | null>(null);

	$effect(() => {
		if (checkboxRef) {
			checkboxRef.indeterminate = (isPartiallySelected ?? false) && !isSelected;
		}
	});
</script>

<button
	{onclick}
	class="relative flex flex-col items-center justify-center p-3 rounded-lg border-2 transition-all duration-150 text-center min-h-[80px] cursor-pointer
		{isDir
			? isRecorded
				? 'border-success/50 bg-success-light hover:border-success'
				: 'border-[var(--border-color)] bg-[var(--bg-secondary)] hover:border-[var(--border-hover)]'
			: isRecorded
				? 'border-success/50 bg-success-light hover:border-success'
				: 'border-[var(--border-color)] bg-[var(--bg-secondary)]/50 hover:border-[var(--border-hover)]'
		}
		{isSelected ? 'ring-2 ring-selected/50' : ''}"
	data-path={node.path}
>
	<!-- Selection checkbox (directories only) -->
	{#if isDir}
		<div
			class="absolute top-1 right-1 z-10"
			role="presentation"
			onclick={(e: MouseEvent) => e.stopPropagation()}
		>
			<input
				bind:this={checkboxRef}
				type="checkbox"
				checked={isSelected}
				onchange={(e) => oncheckchange(e.currentTarget.checked)}
				class="w-4 h-4 rounded accent-[var(--color-primary)] cursor-pointer"
			/>
		</div>
	{/if}

	{#if isDir}
		<svg
			class="mb-1 {isRecorded ? 'text-success' : 'text-[var(--text-muted)]'}"
			xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"
			fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
		>
			<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
		</svg>
	{:else}
		<svg
			class="mb-1 {isRecorded ? 'text-success' : 'text-[var(--text-muted)]'}"
			xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24"
			fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
		>
			<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/>
			<path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>
			<path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
		</svg>
	{/if}

	<span class="text-xs leading-tight break-all max-w-full {isRecorded ? 'text-success' : 'text-[var(--text-primary)]'}">
		{node.name}
	</span>
	{#if isDir && soundCount !== undefined}
		<span class="text-[10px] text-[var(--text-muted)] -mt-0.5">({soundCount})</span>
	{/if}
	{#if showPath && !isDir}
		<span class="text-[10px] leading-tight break-all max-w-full text-[var(--text-muted)] mt-0.5">
			{node.path.replace('minecraft/sounds/', '')}
		</span>
	{/if}

	{#if node.isLongSound}
		<span class="absolute bottom-1 right-1 text-[10px] px-1 py-0.5 rounded bg-warning/20 text-warning font-medium">
			LONG
		</span>
	{/if}
</button>
