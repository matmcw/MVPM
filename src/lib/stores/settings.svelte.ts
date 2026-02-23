import * as api from '$lib/utils/api';
import type { Settings } from '$lib/utils/api';

const defaultSettings: Settings = {
	record_key: 'Space',
	microphone_id: null,
	auto_play_original: true,
	single_recording_mode: false,
	packs_folder: null,
	theme: 'light',
	window_width: null,
	window_height: null,
	window_x: null,
	window_y: null,
};

let settings = $state<Settings>({ ...defaultSettings });
let loaded = $state(false);

export const settingsStore = {
	get current() { return settings; },
	get loaded() { return loaded; },

	get theme() { return settings.theme; },
	get recordKey() { return settings.record_key; },
	get autoPlayOriginal() { return settings.auto_play_original; },
	get singleRecordingMode() { return settings.single_recording_mode; },
	get microphoneId() { return settings.microphone_id; },

	async load() {
		try {
			settings = await api.getSettings();
			loaded = true;
		} catch {
			settings = { ...defaultSettings };
			loaded = true;
		}
	},

	async save(updates: Partial<Settings>) {
		settings = { ...settings, ...updates };
		await api.saveSettings(settings);
	},

	async toggleTheme() {
		const newTheme = settings.theme === 'light' ? 'dark' : 'light';
		await this.save({ theme: newTheme });
	},
};
