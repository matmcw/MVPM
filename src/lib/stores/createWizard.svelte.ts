let version = $state<string | null>(null);
let name = $state('');
let description = $state('');
let iconPath = $state<string | null>(null);

export const createWizardStore = {
	get version() { return version; },
	set version(v: string | null) { version = v; },

	get name() { return name; },
	set name(v: string) { name = v; },

	get description() { return description; },
	set description(v: string) { description = v; },

	get iconPath() { return iconPath; },
	set iconPath(v: string | null) { iconPath = v; },

	clear() {
		version = null;
		name = '';
		description = '';
		iconPath = null;
	},
};
