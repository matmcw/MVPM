<script lang="ts">
	import type { SoundNode } from '$lib/utils/api';

	interface Props {
		node: SoundNode;
		isRecorded: boolean;
		isSelected: boolean;
		onclick: () => void;
		ondblclick?: () => void;
	}

	let { node, isRecorded, isSelected, onclick, ondblclick }: Props = $props();

	const isDir = node.nodeType === 'directory';
</script>

<button
	{onclick}
	ondblclick={ondblclick}
	class="relative flex flex-col items-center justify-center p-3 rounded-lg border-2 transition-all duration-150 text-center min-h-[80px] cursor-pointer
		{isSelected
			? 'border-selected bg-selected-light'
			: isDir
				? isRecorded
					? 'border-success/50 bg-success-light hover:border-success'
					: 'border-[var(--border-color)] bg-[var(--bg-secondary)] hover:border-[var(--border-hover)]'
				: isRecorded
					? 'border-success/50 bg-success-light hover:border-success'
					: 'border-[var(--border-color)] bg-[var(--bg-primary)] hover:border-[var(--border-hover)]'
		}"
>
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

	{#if node.isLongSound}
		<span class="absolute top-1 right-1 text-[10px] px-1 py-0.5 rounded bg-warning/20 text-warning font-medium">
			LONG
		</span>
	{/if}
</button>
