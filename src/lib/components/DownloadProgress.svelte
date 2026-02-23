<script lang="ts">
	import { versionsStore } from '$lib/stores/versions.svelte';

	interface Props {
		open: boolean;
		versionId: string;
		onclose?: () => void;
		onsuccess?: () => void;
	}

	let { open, versionId, onclose, onsuccess }: Props = $props();
	let error = $state<string | null>(null);

	async function startDownload() {
		error = null;
		const success = await versionsStore.downloadVersion(versionId);
		if (success) {
			onsuccess?.();
		} else {
			error = versionsStore.error;
		}
	}

	function retry() {
		startDownload();
	}

	$effect(() => {
		if (open && versionId) {
			startDownload();
		}
	});

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}
</script>

{#if open}
	<div class="fixed inset-0 z-50 flex items-center justify-center">
		<div class="absolute inset-0 bg-black/50" role="presentation"></div>

		<div class="relative bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl shadow-lg p-6 max-w-lg w-full mx-4">
			<h3 class="text-lg font-semibold mb-4">Downloading sounds for {versionId}</h3>

			{#if versionsStore.downloading}
				{@const progress = versionsStore.downloadProgress}
				{#if progress}
					<div class="space-y-3">
						<div class="flex justify-between text-sm text-[var(--text-secondary)]">
							<span>{progress.downloadedFiles} / {progress.totalFiles} files</span>
							<span>{formatBytes(progress.downloadedBytes)} / {formatBytes(progress.totalBytes)}</span>
						</div>

						<div class="w-full bg-[var(--bg-tertiary)] rounded-full h-3">
							<div
								class="bg-primary rounded-full h-3 transition-all duration-300"
								style="width: {progress.totalFiles > 0 ? (progress.downloadedFiles / progress.totalFiles) * 100 : 0}%"
							></div>
						</div>

						<p class="text-sm text-[var(--text-muted)] truncate">
							{progress.currentFile}
						</p>
					</div>
				{:else}
					<p class="text-[var(--text-secondary)]">Preparing download...</p>
				{/if}
			{:else if error}
				<div class="space-y-4">
					<p class="text-danger">{error}</p>
					<div class="flex gap-3 justify-end">
						<button
							onclick={onclose}
							class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors"
						>
							Exit
						</button>
						<button
							onclick={retry}
							class="px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors"
						>
							Retry
						</button>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}
