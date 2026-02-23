<script lang="ts">
	import helpMd from '$lib/assets/help.md?raw';

	interface MarkdownBlock {
		type: 'h1' | 'h2' | 'p' | 'ol' | 'ul';
		content: string;
		items?: string[];
	}

	function parseMarkdown(md: string): MarkdownBlock[] {
		const lines = md.split('\n');
		const blocks: MarkdownBlock[] = [];
		let i = 0;

		while (i < lines.length) {
			const line = lines[i];

			if (line.startsWith('## ')) {
				blocks.push({ type: 'h2', content: line.slice(3) });
				i++;
			} else if (line.startsWith('# ')) {
				blocks.push({ type: 'h1', content: line.slice(2) });
				i++;
			} else if (/^\d+\.\s/.test(line)) {
				const items: string[] = [];
				while (i < lines.length && /^\d+\.\s/.test(lines[i])) {
					items.push(lines[i].replace(/^\d+\.\s/, ''));
					i++;
				}
				blocks.push({ type: 'ol', content: '', items });
			} else if (line.startsWith('- ')) {
				const items: string[] = [];
				while (i < lines.length && lines[i].startsWith('- ')) {
					items.push(lines[i].slice(2));
					i++;
				}
				blocks.push({ type: 'ul', content: '', items });
			} else if (line.trim()) {
				blocks.push({ type: 'p', content: line.trim() });
				i++;
			} else {
				i++;
			}
		}

		return blocks;
	}

	function formatInline(text: string): string {
		return text
			.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
			.replace(/\*(.+?)\*/g, '<em>$1</em>')
			.replace(/`(.+?)`/g, '<code class="bg-[var(--bg-tertiary)] px-1 rounded text-sm">$1</code>');
	}

	const blocks = parseMarkdown(helpMd);
</script>

<div class="max-w-3xl mx-auto p-6">
	<div class="flex items-center gap-3 mb-6">
		<button
			onclick={() => history.back()}
			class="text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<line x1="19" y1="12" x2="5" y2="12"/>
				<polyline points="12 19 5 12 12 5"/>
			</svg>
		</button>
		<h1 class="text-2xl font-bold">Help</h1>
	</div>

	<div class="space-y-4">
		{#each blocks as block}
			{#if block.type === 'h1'}
				<h2 class="text-lg font-semibold mt-8 first:mt-0 mb-1 text-primary">{block.content}</h2>
			{:else if block.type === 'h2'}
				<p class="font-medium text-[var(--text-primary)] mt-6 mb-1">{block.content}</p>
			{:else if block.type === 'p'}
				<p class="text-[var(--text-secondary)]">{@html formatInline(block.content)}</p>
			{:else if block.type === 'ol'}
				<ol class="list-decimal list-inside space-y-1 ml-2 text-[var(--text-secondary)]">
					{#each block.items ?? [] as item}
						<li>{@html formatInline(item)}</li>
					{/each}
				</ol>
			{:else if block.type === 'ul'}
				<ul class="list-disc list-inside space-y-1 ml-2 text-[var(--text-secondary)]">
					{#each block.items ?? [] as item}
						<li>{@html formatInline(item)}</li>
					{/each}
				</ul>
			{/if}
		{/each}
	</div>
</div>
