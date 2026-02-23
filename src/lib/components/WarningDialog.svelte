<script lang="ts">
	interface Props {
		open: boolean;
		title: string;
		message: string;
		confirmText?: string;
		cancelText?: string;
		dangerousConfirmText?: string;
		onconfirm?: () => void;
		oncancel?: () => void;
		ondangerous?: () => void;
	}

	let {
		open,
		title,
		message,
		confirmText = 'Confirm',
		cancelText = 'Cancel',
		dangerousConfirmText,
		onconfirm,
		oncancel,
		ondangerous,
	}: Props = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (e.key === 'Escape') {
			oncancel?.();
		} else if (e.key === 'Enter') {
			onconfirm?.();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div class="fixed inset-0 z-50 flex items-center justify-center">
		<!-- Backdrop -->
		<div
			class="absolute inset-0 bg-black/50"
			onclick={oncancel}
			role="presentation"
		></div>

		<!-- Dialog -->
		<div class="relative bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl shadow-lg p-6 max-w-md w-full mx-4">
			<h3 class="text-lg font-semibold mb-2">{title}</h3>
			<p class="text-[var(--text-secondary)] mb-6">{message}</p>

			<div class="flex gap-3 justify-end">
				{#if oncancel}
					<button
						onclick={oncancel}
						class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors"
					>
						{cancelText}
					</button>
				{/if}
				{#if ondangerous && dangerousConfirmText}
					<button
						onclick={ondangerous}
						class="px-4 py-2 rounded-lg bg-danger text-white hover:bg-danger-hover transition-colors"
					>
						{dangerousConfirmText}
					</button>
				{/if}
				{#if onconfirm}
					<button
						onclick={onconfirm}
						class="px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors"
					>
						{confirmText}
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}
