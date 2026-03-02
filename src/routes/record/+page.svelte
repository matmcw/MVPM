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
	let progressBarRef = $state<HTMLElement | null>(null);

	const currentSound = $derived(recordingStore.currentSound);
	const isCurrentRecorded = $derived(
		currentSound
			? recordedInSession.has(currentSound.path) ||
				(packStore.currentPack?.recordedSounds.includes(currentSound.path) ?? false)
			: false
	);

	function displayName(name: string): string {
		let n = name.replace(/\.ogg$/i, '');
		if (recordingStore.singleMode) n = n.replace(/\d+$/, '');
		return n;
	}

	function displayPath(path: string): string {
		return path.replace('minecraft/sounds/', '').replace(/\.ogg$/i, '');
	}

	function isSoundRecorded(index: number): boolean {
		const sound = recordingStore.sounds[index];
		if (!sound) return false;
		return recordedInSession.has(sound.path) ||
			(packStore.currentPack?.recordedSounds.includes(sound.path) ?? false);
	}

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
			ctx.clearRect(0, 0, w, h);

			ctx.lineWidth = 2;
			ctx.strokeStyle = isRecording ? '#ef4444' : isCurrentRecorded ? '#22c55e' : '#94a3b8';
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

			// Auto-advance to next unrecorded (or next sound if all recorded)
			if (recordingStore.autoSkip) {
				const allRecorded = [
					...(packStore.currentPack?.recordedSounds ?? []),
					...recordedInSession,
				];
				const found = recordingStore.nextUnrecorded(allRecorded);
				if (!found) {
					recordingStore.next();
				}
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

	async function deleteCurrentRecording() {
		if (!currentSound) return;
		saving = true;
		try {
			await api.deleteRecording(recordingStore.packId, currentSound.path);
			if (settingsStore.singleRecordingMode && currentSound.variants) {
				for (const v of currentSound.variants) {
					if (v !== currentSound.path) {
						await api.deleteRecording(recordingStore.packId, v);
					}
				}
			}
			const newSet = new Set(recordedInSession);
			newSet.delete(currentSound.path);
			if (settingsStore.singleRecordingMode && currentSound.variants) {
				currentSound.variants.forEach((v) => newSet.delete(v));
			}
			recordedInSession = newSet;
			await packStore.refreshCurrentPack();
		} catch (e) {
			console.error('Failed to delete recording:', e);
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

	function goToSound(index: number) {
		if (isRecording) return;
		recordingStore.setAutoSkip(false);
		recordingStore.goToIndex(index);
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

	// Auto-scroll progress bar to active segment
	$effect(() => {
		const idx = recordingStore.currentIndex;
		if (progressBarRef) {
			const segment = progressBarRef.querySelector(`[data-index="${idx}"]`);
			if (segment) {
				segment.scrollIntoView({ inline: 'center', behavior: 'smooth', block: 'nearest' });
			}
		}
	});
</script>

<svelte:window onkeydown={handleKeyDown} onkeyup={handleKeyUp} />

<div class="flex flex-col h-full" style="user-select: none;">
	{#if currentSound}
		<!-- Header: Done button only -->
		<div class="px-4 py-2 border-b border-[var(--border-color)] bg-[var(--bg-secondary)] flex items-center justify-end">
			<button
				onclick={handleDone}
				disabled={isRecording}
				class="px-5 py-2.5 rounded-lg bg-cta text-white hover:bg-cta-hover transition-colors disabled:opacity-50 font-medium"
			>
				Done
			</button>
		</div>

		<!-- Center content -->
		<div class="flex-1 overflow-auto flex flex-col items-center justify-center px-8 py-6">
			<!-- Sound info -->
			<h2 class="text-3xl font-bold leading-tight text-center mb-1">{displayName(currentSound.name)}</h2>
			<p class="text-sm text-[var(--text-muted)] text-center mb-6">
				{displayPath(currentSound.path)}
				{#if currentSound.soundEvent}
					&middot; {currentSound.soundEvent}
				{/if}
			</p>

			<!-- Waveform -->
			<canvas
				bind:this={canvasEl}
				width="800"
				height="200"
				class="w-full max-w-3xl rounded-lg border border-[var(--border-color)] bg-[var(--bg-secondary)] mb-6"
			></canvas>

			<!-- Timer -->
			<p class="text-4xl font-mono mb-3 {isRecording ? 'text-recording' : 'text-[var(--text-primary)]'}">
				{formatTime(recordingTime)}
			</p>

			<!-- Recording hint -->
			<p class="text-base text-[var(--text-muted)] mb-6">
				{#if isRecording}
					Release {settingsStore.recordKey} to stop
				{:else if saving}
					Saving...
				{:else}
					Hold {settingsStore.recordKey} to record
				{/if}
			</p>

			<!-- Playback controls -->
			<div class="flex items-center gap-3">
				<button
					onclick={playOriginal}
					disabled={isRecording}
					class="px-5 py-3 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors text-base disabled:opacity-50"
				>
					Play Original
				</button>
				<button
					onclick={playRecording}
					disabled={isRecording || !isCurrentRecorded}
					class="px-5 py-3 rounded-lg border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors text-base disabled:opacity-50"
				>
					Play Recording
				</button>
				<button
					onclick={deleteCurrentRecording}
					disabled={isRecording || !isCurrentRecorded || saving}
					class="px-5 py-3 rounded-lg border border-red-400/30 text-red-400 hover:text-red-300 hover:bg-red-400/10 transition-colors text-base disabled:opacity-30"
				>
					Delete
				</button>
			</div>
		</div>

		<!-- Bottom section -->
		<div class="border-t border-[var(--border-color)] bg-[var(--bg-secondary)]">
			<!-- Navigation + Auto-Advance compact island -->
			<div class="flex items-center justify-center px-2 py-2">
				<div class="flex items-center">
					<button
						onclick={handlePrevious}
						disabled={recordingStore.currentIndex === 0 || isRecording}
						class="px-3 self-stretch rounded-l-lg hover:bg-[var(--bg-tertiary)] transition-colors disabled:opacity-30 flex items-center"
						aria-label="Previous sound"
					>
						<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<polyline points="15 18 9 12 15 6"/>
						</svg>
					</button>

					<div class="flex flex-col items-center gap-1 py-1 px-6">
						<span class="text-sm text-[var(--text-secondary)]">{recordingStore.progress}</span>
						<div class="flex items-center gap-2">
							<button
								type="button"
								onclick={() => recordingStore.setAutoSkip(!recordingStore.autoSkip)}
								disabled={isRecording}
								class="w-9 h-5 rounded-full transition-colors relative shrink-0 disabled:opacity-50 {recordingStore.autoSkip ? 'bg-primary' : 'bg-[var(--bg-tertiary)]'}"
							>
								<span class="block h-4 w-4 rounded-full bg-white shadow transition-transform absolute top-[2px] left-[2px] {recordingStore.autoSkip ? 'translate-x-full' : ''}"></span>
							</button>
							<span class="text-xs text-[var(--text-muted)]">Auto-Advance</span>
						</div>
					</div>

					<button
						onclick={handleNext}
						disabled={recordingStore.currentIndex >= recordingStore.total - 1 || isRecording}
						class="px-3 self-stretch rounded-r-lg hover:bg-[var(--bg-tertiary)] transition-colors disabled:opacity-30 flex items-center"
						aria-label="Next sound"
					>
						<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<polyline points="9 18 15 12 9 6"/>
						</svg>
					</button>
				</div>
			</div>

			<!-- Progress bar -->
			<div class="overflow-x-auto" bind:this={progressBarRef}>
				<div class="flex border border-[var(--border-color)] rounded overflow-hidden" style="min-width: fit-content;">
					{#each recordingStore.sounds as sound, i (sound.path)}
						<button
							data-index={i}
							class="h-8 min-w-[20px] flex-1 relative group transition-all cursor-pointer border-r border-[var(--border-color)] last:border-r-0 hover:brightness-125
								{i === recordingStore.currentIndex ? 'ring-2 ring-inset ring-white/50' : ''}"
							style:background-color={
								i === recordingStore.currentIndex && isRecording ? '#ef4444'
								: isSoundRecorded(i) ? '#22c55e'
								: 'var(--bg-tertiary)'
							}
							onclick={() => goToSound(i)}
						>
							<!-- Tooltip -->
							<span class="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 px-2 py-1 text-[10px] bg-[var(--bg-primary)] border border-[var(--border-color)] rounded shadow-lg whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-10">
								{displayName(sound.name)} &middot; {displayPath(sound.path)}
							</span>
						</button>
					{/each}
				</div>
			</div>
		</div>
	{:else}
		<div class="flex flex-col h-full items-center justify-center p-6">
			<p class="text-[var(--text-muted)]">No sounds to record.</p>
			<button
				onclick={() => history.back()}
				class="mt-4 px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors"
			>
				Go Back
			</button>
		</div>
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
