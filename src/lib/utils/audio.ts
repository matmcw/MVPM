import { convertFileSrc } from '@tauri-apps/api/core';

let currentAudio: HTMLAudioElement | null = null;

export function playLocalFile(filePath: string): HTMLAudioElement {
	stopPlayback();
	const url = convertFileSrc(filePath);
	currentAudio = new Audio(url);
	currentAudio.play();
	return currentAudio;
}

export function stopPlayback() {
	if (currentAudio) {
		currentAudio.pause();
		currentAudio.currentTime = 0;
		currentAudio = null;
	}
}

export function isPlaying(): boolean {
	return currentAudio !== null && !currentAudio.paused;
}

// --- Recording ---

let mediaRecorder: MediaRecorder | null = null;
let audioContext: AudioContext | null = null;
let analyserNode: AnalyserNode | null = null;
let mediaStream: MediaStream | null = null;
let recordingChunks: Blob[] = [];

export async function initRecording(deviceId?: string): Promise<AnalyserNode> {
	const constraints: MediaStreamConstraints = {
		audio: deviceId
			? { deviceId: { exact: deviceId }, sampleRate: 44100, channelCount: 1 }
			: { sampleRate: 44100, channelCount: 1 },
	};

	mediaStream = await navigator.mediaDevices.getUserMedia(constraints);
	audioContext = new AudioContext({ sampleRate: 44100 });
	const source = audioContext.createMediaStreamSource(mediaStream);
	analyserNode = audioContext.createAnalyser();
	analyserNode.fftSize = 2048;
	source.connect(analyserNode);

	return analyserNode;
}

export function startRecording(): void {
	if (!mediaStream) throw new Error('Recording not initialized. Call initRecording first.');

	recordingChunks = [];
	mediaRecorder = new MediaRecorder(mediaStream, {
		mimeType: 'audio/webm;codecs=opus',
	});

	mediaRecorder.ondataavailable = (e) => {
		if (e.data.size > 0) {
			recordingChunks.push(e.data);
		}
	};

	mediaRecorder.start(100);
}

export async function stopRecording(): Promise<ArrayBuffer> {
	return new Promise((resolve, reject) => {
		if (!mediaRecorder || mediaRecorder.state === 'inactive') {
			reject(new Error('Not recording'));
			return;
		}

		mediaRecorder.onstop = async () => {
			try {
				const blob = new Blob(recordingChunks, { type: 'audio/webm' });
				const wavBuffer = await blobToWav(blob);
				resolve(wavBuffer);
			} catch (e) {
				reject(e);
			}
		};

		mediaRecorder.stop();
	});
}

export function getAnalyser(): AnalyserNode | null {
	return analyserNode;
}

export function cleanupRecording(): void {
	if (mediaRecorder && mediaRecorder.state !== 'inactive') {
		mediaRecorder.stop();
	}
	mediaRecorder = null;

	if (mediaStream) {
		mediaStream.getTracks().forEach((track) => track.stop());
		mediaStream = null;
	}

	if (audioContext) {
		audioContext.close();
		audioContext = null;
	}

	analyserNode = null;
	recordingChunks = [];
}

export function isRecording(): boolean {
	return mediaRecorder !== null && mediaRecorder.state === 'recording';
}

// Convert WebM blob to WAV ArrayBuffer
async function blobToWav(blob: Blob): Promise<ArrayBuffer> {
	const arrayBuffer = await blob.arrayBuffer();
	const audioCtx = new OfflineAudioContext(1, 44100 * 300, 44100);
	const audioBuffer = await audioCtx.decodeAudioData(arrayBuffer);

	const numChannels = 1;
	const sampleRate = 44100;
	const samples = audioBuffer.getChannelData(0);
	const numSamples = samples.length;

	const wavBuffer = new ArrayBuffer(44 + numSamples * 2);
	const view = new DataView(wavBuffer);

	// WAV header
	writeString(view, 0, 'RIFF');
	view.setUint32(4, 36 + numSamples * 2, true);
	writeString(view, 8, 'WAVE');
	writeString(view, 12, 'fmt ');
	view.setUint32(16, 16, true);
	view.setUint16(20, 1, true);
	view.setUint16(22, numChannels, true);
	view.setUint32(24, sampleRate, true);
	view.setUint32(28, sampleRate * numChannels * 2, true);
	view.setUint16(32, numChannels * 2, true);
	view.setUint16(34, 16, true);
	writeString(view, 36, 'data');
	view.setUint32(40, numSamples * 2, true);

	// Convert float samples to 16-bit PCM
	let offset = 44;
	for (let i = 0; i < numSamples; i++) {
		const s = Math.max(-1, Math.min(1, samples[i]));
		view.setInt16(offset, s < 0 ? s * 0x8000 : s * 0x7fff, true);
		offset += 2;
	}

	return wavBuffer;
}

function writeString(view: DataView, offset: number, str: string): void {
	for (let i = 0; i < str.length; i++) {
		view.setUint8(offset + i, str.charCodeAt(i));
	}
}

// Get available audio input devices
export async function getAudioDevices(): Promise<MediaDeviceInfo[]> {
	const devices = await navigator.mediaDevices.enumerateDevices();
	return devices.filter((d) => d.kind === 'audioinput');
}
