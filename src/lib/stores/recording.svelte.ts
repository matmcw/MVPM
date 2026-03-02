import type { SoundNode } from '$lib/utils/api';

let sounds = $state<SoundNode[]>([]);
let currentIndex = $state(0);
let autoSkip = $state(false);
let packId = $state('');
let versionId = $state('');
let singleMode = $state(false);

export const recordingStore = {
	get sounds() { return sounds; },
	get currentIndex() { return currentIndex; },
	get currentSound(): SoundNode | null { return sounds[currentIndex] ?? null; },
	get autoSkip() { return autoSkip; },
	get packId() { return packId; },
	get versionId() { return versionId; },
	get singleMode() { return singleMode; },
	get total() { return sounds.length; },

	get progress() {
		return `Recording ${currentIndex + 1} of ${sounds.length}`;
	},

	setup(
		selectedSounds: SoundNode[],
		pack: string,
		version: string,
		isSingleMode: boolean = false,
	) {
		sounds = selectedSounds;
		currentIndex = 0;
		autoSkip = false;
		packId = pack;
		versionId = version;
		singleMode = isSingleMode;
	},

	next() {
		if (currentIndex < sounds.length - 1) {
			currentIndex++;
		}
	},

	previous() {
		if (currentIndex > 0) {
			currentIndex--;
			autoSkip = false;
		}
	},

	goToIndex(index: number) {
		if (index >= 0 && index < sounds.length) {
			currentIndex = index;
		}
	},

	nextUnrecorded(recordedSounds: string[]) {
		for (let i = currentIndex + 1; i < sounds.length; i++) {
			if (!recordedSounds.includes(sounds[i].path)) {
				currentIndex = i;
				return true;
			}
		}
		// Wrap around from beginning
		for (let i = 0; i < currentIndex; i++) {
			if (!recordedSounds.includes(sounds[i].path)) {
				currentIndex = i;
				return true;
			}
		}
		return false;
	},

	setAutoSkip(value: boolean) {
		autoSkip = value;
	},

	manualNavigate(direction: 'next' | 'previous') {
		autoSkip = false;
		if (direction === 'next') {
			this.next();
		} else {
			this.previous();
		}
	},

	hasUnrecorded(recordedSounds: string[]): boolean {
		return sounds.some((s) => !recordedSounds.includes(s.path));
	},

	getUnrecordedCount(recordedSounds: string[]): number {
		return sounds.filter((s) => !recordedSounds.includes(s.path)).length;
	},

	clear() {
		sounds = [];
		currentIndex = 0;
		autoSkip = false;
		packId = '';
		versionId = '';
		singleMode = false;
	},
};
