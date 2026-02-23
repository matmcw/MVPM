<script lang="ts">
	interface Props {
		value: string;
		placeholder?: string;
		oninput: (value: string) => void;
	}

	let { value, placeholder = 'Search sounds...', oninput }: Props = $props();
</script>

<div class="relative">
	<svg
		class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
		xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"
		fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
	>
		<circle cx="11" cy="11" r="8"/>
		<line x1="21" y1="21" x2="16.65" y2="16.65"/>
	</svg>
	<input
		type="text"
		{value}
		{placeholder}
		oninput={(e) => oninput(e.currentTarget.value)}
		class="w-full pl-9 pr-4 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-primary transition-colors"
	/>
	{#if value}
		<button
			onclick={() => oninput('')}
			class="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<line x1="18" y1="6" x2="6" y2="18"/>
				<line x1="6" y1="6" x2="18" y2="18"/>
			</svg>
		</button>
	{/if}
</div>
