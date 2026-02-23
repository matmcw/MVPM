<script lang="ts">
	import { onMount } from 'svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { getAudioDevices } from '$lib/utils/audio';
	import { open } from '@tauri-apps/plugin-dialog';
	import { goto } from '$app/navigation';

	let devices = $state<MediaDeviceInfo[]>([]);
	let capturingKey = $state(false);
	let saved = $state(false);

	onMount(async () => {
		try {
			devices = await getAudioDevices();
		} catch {
			// Mic permission may not be granted yet
		}
	});

	function captureKey(e: KeyboardEvent) {
		if (!capturingKey) return;
		e.preventDefault();
		settingsStore.save({ record_key: e.code });
		capturingKey = false;
		flashSaved();
	}

	async function selectPacksFolder() {
		const result = await open({
			multiple: false,
			directory: true,
		});
		if (result) {
			await settingsStore.save({ packs_folder: result as string });
			flashSaved();
		}
	}

	function flashSaved() {
		saved = true;
		setTimeout(() => (saved = false), 2000);
	}

	async function handleToggle(key: string, value: boolean) {
		await settingsStore.save({ [key]: value });
		flashSaved();
	}

	async function handleMicChange(deviceId: string) {
		await settingsStore.save({ microphone_id: deviceId || null });
		flashSaved();
	}
</script>

<svelte:window onkeydown={captureKey} />

<div class="max-w-2xl mx-auto p-6">
	<div class="flex items-center gap-3 mb-6">
		<button
			onclick={() => goto('/')}
			class="text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<line x1="19" y1="12" x2="5" y2="12"/>
				<polyline points="12 19 5 12 12 5"/>
			</svg>
		</button>
		<h1 class="text-2xl font-bold">Settings</h1>
		{#if saved}
			<span class="text-sm text-success ml-auto">Saved</span>
		{/if}
	</div>

	<div class="space-y-6">
		<!-- Record key -->
		<div class="flex items-center justify-between py-3 border-b border-[var(--border-color)]">
			<div>
				<h3 class="font-medium">Record Key</h3>
				<p class="text-sm text-[var(--text-muted)]">Hold this key to record</p>
			</div>
			<button
				onclick={() => (capturingKey = !capturingKey)}
				class="px-4 py-2 rounded-lg border border-[var(--border-color)] font-mono text-sm min-w-[120px] text-center transition-colors
					{capturingKey ? 'border-primary bg-selected-light' : 'hover:bg-[var(--bg-tertiary)]'}"
			>
				{capturingKey ? 'Press a key...' : settingsStore.recordKey}
			</button>
		</div>

		<!-- Microphone -->
		<div class="flex items-center justify-between py-3 border-b border-[var(--border-color)]">
			<div>
				<h3 class="font-medium">Microphone</h3>
				<p class="text-sm text-[var(--text-muted)]">Audio input device</p>
			</div>
			<select
				value={settingsStore.microphoneId ?? ''}
				onchange={(e) => handleMicChange(e.currentTarget.value)}
				class="px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] max-w-[200px]"
			>
				<option value="">Default</option>
				{#each devices as device}
					<option value={device.deviceId}>{device.label || `Microphone ${device.deviceId.slice(0, 8)}`}</option>
				{/each}
			</select>
		</div>

		<!-- Auto-play -->
		<div class="flex items-center justify-between py-3 border-b border-[var(--border-color)]">
			<div>
				<h3 class="font-medium">Auto-play Original Sound</h3>
				<p class="text-sm text-[var(--text-muted)]">Play the original sound when entering each recording step</p>
			</div>
			<label class="relative inline-flex items-center cursor-pointer">
				<input
					type="checkbox"
					checked={settingsStore.autoPlayOriginal}
					onchange={(e) => handleToggle('auto_play_original', e.currentTarget.checked)}
					class="sr-only peer"
				/>
				<div class="w-11 h-6 bg-[var(--bg-tertiary)] peer-focus:outline-none rounded-full peer peer-checked:bg-primary transition-colors after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
			</label>
		</div>

		<!-- Single recording mode -->
		<div class="flex items-center justify-between py-3 border-b border-[var(--border-color)]">
			<div>
				<h3 class="font-medium">Single Recording Mode</h3>
				<p class="text-sm text-[var(--text-muted)]">Record once per sound event, duplicate to all variants</p>
			</div>
			<label class="relative inline-flex items-center cursor-pointer">
				<input
					type="checkbox"
					checked={settingsStore.singleRecordingMode}
					onchange={(e) => handleToggle('single_recording_mode', e.currentTarget.checked)}
					class="sr-only peer"
				/>
				<div class="w-11 h-6 bg-[var(--bg-tertiary)] peer-focus:outline-none rounded-full peer peer-checked:bg-primary transition-colors after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
			</label>
		</div>

		<!-- Packs folder -->
		<div class="flex items-center justify-between py-3 border-b border-[var(--border-color)]">
			<div>
				<h3 class="font-medium">Packs Folder</h3>
				<p class="text-sm text-[var(--text-muted)] truncate max-w-[300px]">
					{settingsStore.current.packs_folder ?? 'Default (app folder/packs/)'}
				</p>
			</div>
			<button
				onclick={selectPacksFolder}
				class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors text-sm"
			>
				Change
			</button>
		</div>

		<!-- Theme -->
		<div class="flex items-center justify-between py-3 border-b border-[var(--border-color)]">
			<div>
				<h3 class="font-medium">Theme</h3>
				<p class="text-sm text-[var(--text-muted)]">Switch between light and dark mode</p>
			</div>
			<button
				onclick={() => settingsStore.toggleTheme()}
				class="px-4 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors text-sm capitalize"
			>
				{settingsStore.theme}
			</button>
		</div>
	</div>
</div>
