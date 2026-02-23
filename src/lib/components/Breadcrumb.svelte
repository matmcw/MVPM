<script lang="ts">
	interface Crumb {
		name: string;
		path: string[];
	}

	interface Props {
		crumbs: Crumb[];
		onnavigate: (path: string[]) => void;
	}

	let { crumbs, onnavigate }: Props = $props();
</script>

<nav class="flex items-center gap-1 text-sm text-[var(--text-secondary)] overflow-x-auto">
	{#each crumbs as crumb, i}
		{#if i > 0}
			<span class="text-[var(--text-muted)]">/</span>
		{/if}
		{#if i < crumbs.length - 1}
			<button
				onclick={() => onnavigate(crumb.path)}
				class="hover:text-[var(--text-primary)] hover:underline px-1 py-0.5 rounded whitespace-nowrap"
			>
				{crumb.name}
			</button>
		{:else}
			<span class="text-[var(--text-primary)] font-medium px-1 py-0.5 whitespace-nowrap">
				{crumb.name}
			</span>
		{/if}
	{/each}
</nav>
