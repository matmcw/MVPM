<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { recordingStore } from '$lib/stores/recording.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { packStore } from '$lib/stores/pack.svelte';
	import * as audio from '$lib/utils/audio';
	import * as api from '$lib/utils/api';
	import WarningDialog from '$lib/components/WarningDialog.svelte';

	let isRecording = $state(false);
	let recordingTime = $state(0);
	let timer: ReturnType<typeof setInterval> | null = null;
	let analyser = $state<AnalyserNode | null>(null);
	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let animFrameId = $state(0);
	let showExitWarning = $state(false);
	let saving = $state(false);
	let recordedInSession = $state<Set<string>>(new Set());

	const currentSound = $derived(recordingStore.currentSound);
	const isCurrentRecorded = $derived(
		currentSound
			? recordedInSession.has(currentSound.path) ||
				(packStore.currentPack?.recordedSounds.includes(currentSound.path) ?? false)
			: false
	);

	const statusColor = $derived(
		isRecording ? 'bg-recording' : isCurrentRecorded ? 'bg-success' : 'bg-[var(--bg-tertiary)]'
	);

	onMount(async () => {
		if (recordingStore.sounds.length === 0) {
			goto('/');
			return;
		}

		try {
			analyser = await audio.initRecording(settingsStore.microphoneId ?? undefined);
			drawWaveform();
		} catch (e) {
			console.error('Failed to init recording:', e);
		}

		if (settingsStore.autoPlayOriginal && currentSound) {
			playOriginal();
		}
	});

	onDestroy(() => {
		audio.cleanupRecording();
		audio.stopPlayback();
		if (timer) clearInterval(timer);
		if (animFrameId) cancelAnimationFrame(animFrameId);
	});

	function drawWaveform() {
		if (!canvasEl || !analyser) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;

		const bufferLength = analyser.frequencyBinCount;
		const dataArray = new Uint8Array(bufferLength);

		function draw() {
			if (!ctx || !canvasEl || !analyser) return;
			animFrameId = requestAnimationFrame(draw);

			analyser.getByteTimeDomainData(dataArray);

			const w = canvasEl.width;
			const h = canvasEl.height;
			ctx.fillStyle = 'var(--bg-secondary)';
			ctx.fillRect(0, 0, w, h);

			ctx.lineWidth = 2;
			ctx.strokeStyle = isRecording ? '#ef4444' : '#94a3b8';
			ctx.beginPath();

			const sliceWidth = w / bufferLength;
			let x = 0;

			for (let i = 0; i < bufferLength; i++) {
				const v = dataArray[i] / 128.0;
				const y = (v * h) / 2;

				if (i === 0) ctx.moveTo(x, y);
				else ctx.lineTo(x, y);
				x += sliceWidth;
			}

			ctx.lineTo(w, h / 2);
			ctx.stroke();
		}

		draw();
	}

	async function playOriginal() {
		if (!currentSound) return;
		try {
			const path = await api.getOriginalSoundPath(recordingStore.versionId, currentSound.path);
			audio.playLocalFile(path);
		} catch (e) {
			console.error('Failed to play original:', e);
		}
	}

	async function playRecording() {
		if (!currentSound) return;
		try {
			const path = await api.getPackSoundPath(recordingStore.packId, currentSound.path);
			if (path) {
				audio.playLocalFile(path);
			}
		} catch (e) {
			console.error('Failed to play recording:', e);
		}
	}

	function startRecording() {
		audio.stopPlayback();
		audio.startRecording();
		isRecording = true;
		recordingTime = 0;
		timer = setInterval(() => {
			recordingTime += 10;
		}, 10);
	}

	async function stopRecording() {
		if (!isRecording || !currentSound) return;
		isRecording = false;
		if (timer) {
			clearInterval(timer);
			timer = null;
		}

		saving = true;
		try {
			const wavBuffer = await audio.stopRecording();
			const wavData = Array.from(new Uint8Array(wavBuffer));

			await api.saveRecording(
				recordingStore.packId,
				currentSound.path,
				wavData,
				settingsStore.singleRecordingMode,
				currentSound.variants ?? undefined,
			);

			// Track what we recorded this session
			const newSet = new Set(recordedInSession);
			newSet.add(currentSound.path);
			if (settingsStore.singleRecordingMode && currentSound.variants) {
				currentSound.variants.forEach((v) => newSet.add(v));
			}
			recordedInSession = newSet;

			// Refresh pack metadata
			await packStore.refreshCurrentPack();

			// Auto-skip to next unrecorded
			if (recordingStore.autoSkip) {
				const allRecorded = [
					...(packStore.currentPack?.recordedSounds ?? []),
					...recordedInSession,
				];
				recordingStore.nextUnrecorded(allRecorded);
				if (settingsStore.autoPlayOriginal) {
					setTimeout(playOriginal, 200);
				}
			}
		} catch (e) {
			console.error('Failed to save recording:', e);
		} finally {
			saving = false;
		}
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.code === settingsStore.recordKey && !e.repeat) {
			e.preventDefault();
			if (!isRecording && !saving) {
				startRecording();
			}
		}
	}

	function handleKeyUp(e: KeyboardEvent) {
		if (e.code === settingsStore.recordKey) {
			e.preventDefault();
			if (isRecording) {
				stopRecording();
			}
		}
	}

	function handlePrevious() {
		recordingStore.manualNavigate('previous');
		if (settingsStore.autoPlayOriginal) {
			setTimeout(playOriginal, 200);
		}
	}

	function handleNext() {
		recordingStore.manualNavigate('next');
		if (settingsStore.autoPlayOriginal) {
			setTimeout(playOriginal, 200);
		}
	}

	function handleDone() {
		const allRecorded = [
			...(packStore.currentPack?.recordedSounds ?? []),
			...recordedInSession,
		];
		if (recordingStore.hasUnrecorded(allRecorded)) {
			showExitWarning = true;
		} else {
			exitRecording();
		}
	}

	function handleRecordUnrecorded() {
		showExitWarning = false;
		const allRecorded = [
			...(packStore.currentPack?.recordedSounds ?? []),
			...recordedInSession,
		];
		recordingStore.setAutoSkip(true);
		recordingStore.nextUnrecorded(allRecorded);
		if (settingsStore.autoPlayOriginal) {
			setTimeout(playOriginal, 200);
		}
	}

	function exitRecording() {
		showExitWarning = false;
		recordingStore.clear();
		history.back();
	}

	function formatTime(ms: number): string {
		const seconds = Math.floor(ms / 1000);
		const minutes = Math.floor(seconds / 60);
		const remainingSeconds = seconds % 60;
		const remainingMs = Math.floor((ms % 1000) / 10);
		return `${minutes}:${String(remainingSeconds).padStart(2, '0')}.${String(remainingMs).padStart(2, '0')}`;
	}

	// Auto-play when sound changes
	let lastSoundPath = $state('');
	$effect(() => {
		if (currentSound && currentSound.path !== lastSoundPath) {
			lastSoundPath = currentSound.path;
			if (settingsStore.autoPlayOriginal && !isRecording) {
				playOriginal();
			}
		}
	});
</script>

<svelte:window onkeydown={handleKeyDown} onkeyup={handleKeyUp} />

<div class="flex flex-col h-full items-center justify-center p-6 no-select">
	{#if currentSound}
		<!-- Sound name -->
		<h2 class="text-xl font-bold mb-1 text-center">{currentSound.name}</h2>
		<p class="text-sm text-[var(--text-muted)] mb-1">
			{currentSound.path.replace('minecraft/sounds/', '')}
		</p>
		{#if currentSound.soundEvent}
			<p class="text-xs text-[var(--text-muted)] mb-4">{currentSound.soundEvent}</p>
		{/if}

		<!-- Progress -->
		<p class="text-sm text-[var(--text-secondary)] mb-6">{recordingStore.progress}</p>

		<!-- Status indicator -->
		<div class="w-4 h-4 rounded-full {statusColor} mb-4 transition-colors"></div>

		<!-- Waveform -->
		<canvas
			bind:this={canvasEl}
			width="400"
			height="80"
			class="rounded-lg border border-[var(--border-color)] bg-[var(--bg-secondary)] mb-4"
		></canvas>

		<!-- Timer -->
		<p class="text-2xl font-mono mb-6 {isRecording ? 'text-recording' : 'text-[var(--text-primary)]'}">
			{formatTime(recordingTime)}
		</p>

		<!-- Recording hint -->
		<p class="text-sm text-[var(--text-muted)] mb-6">
			{#if isRecording}
				Release {settingsStore.recordKey} to stop
			{:else if saving}
				Saving...
			{:else}
				Hold {settingsStore.recordKey} to record
			{/if}
		</p>

		<!-- Playback controls -->
		<div class="flex items-center gap-3 mb-6">
			<button
				onclick={playOriginal}
				disabled={isRecording}
				class="px-3 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors text-sm disabled:opacity-50"
			>
				Play Original
			</button>
			<button
				onclick={playRecording}
				disabled={isRecording || !isCurrentRecorded}
				class="px-3 py-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors text-sm disabled:opacity-50"
			>
				Play Recording
			</button>
		</div>

		<!-- Navigation controls -->
		<div class="flex items-center gap-4 mb-6">
			<button
				onclick={handlePrevious}
				disabled={recordingStore.currentIndex === 0 || isRecording}
				class="p-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors disabled:opacity-50"
			>
				<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<polyline points="15 18 9 12 15 6"/>
				</svg>
			</button>

			<!-- Auto-skip toggle -->
			<label class="flex items-center gap-2 text-sm cursor-pointer">
				<input
					type="checkbox"
					checked={recordingStore.autoSkip}
					onchange={(e) => recordingStore.setAutoSkip(e.currentTarget.checked)}
					class="rounded"
				/>
				Auto-skip to next unrecorded
			</label>

			<button
				onclick={handleNext}
				disabled={recordingStore.currentIndex >= recordingStore.total - 1 || isRecording}
				class="p-2 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors disabled:opacity-50"
			>
				<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<polyline points="9 18 15 12 9 6"/>
				</svg>
			</button>
		</div>

		<!-- Done button -->
		<button
			onclick={handleDone}
			disabled={isRecording}
			class="px-6 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors disabled:opacity-50"
		>
			Done
		</button>
	{:else}
		<p class="text-[var(--text-muted)]">No sounds to record.</p>
		<button
			onclick={() => history.back()}
			class="mt-4 px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors"
		>
			Go Back
		</button>
	{/if}
</div>

<WarningDialog
	open={showExitWarning}
	title="Unrecorded sounds"
	message="Some selected sounds were not recorded. Would you like to record them now?"
	confirmText="Yes"
	cancelText="Skip"
	onconfirm={handleRecordUnrecorded}
	oncancel={exitRecording}
/>
