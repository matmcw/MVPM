<script lang="ts">
	import { goto } from '$app/navigation';
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

	<div class="space-y-8">
		<!-- Getting Started -->
		<section>
			<h2 class="text-lg font-semibold mb-3 text-primary">Getting Started</h2>
			<div class="text-[var(--text-secondary)] space-y-2">
				<p>MVPM lets you create custom Minecraft resource packs by recording your own sounds to replace the default game sounds.</p>
				<ol class="list-decimal list-inside space-y-1 ml-2">
					<li>Click <strong>"Create New Pack"</strong> on the home screen.</li>
					<li>Select the Minecraft version you play on.</li>
					<li>Give your pack a name and description.</li>
					<li>Optionally add a pack icon.</li>
					<li>Start recording sounds!</li>
				</ol>
			</div>
		</section>

		<!-- Recording Sounds -->
		<section>
			<h2 class="text-lg font-semibold mb-3 text-primary">Recording Sounds</h2>
			<div class="text-[var(--text-secondary)] space-y-2">
				<p>Navigate the sound categories by double-clicking folder tiles. Select individual sounds or entire categories by clicking, then press <strong>"Record Selected"</strong>.</p>
				<p>In the recording screen:</p>
				<ul class="list-disc list-inside space-y-1 ml-2">
					<li><strong>Hold the record key</strong> (spacebar by default) to record.</li>
					<li><strong>Release</strong> to stop recording. The file is saved automatically.</li>
					<li>Use the <strong>Play Original</strong> button to hear the default sound.</li>
					<li>Use the <strong>Play Recording</strong> button to hear your recording.</li>
					<li>Enable <strong>"Auto-skip"</strong> to automatically move to the next unrecorded sound.</li>
				</ul>
			</div>
		</section>

		<!-- Using Your Pack -->
		<section>
			<h2 class="text-lg font-semibold mb-3 text-primary">Using Your Pack in Minecraft</h2>
			<div class="text-[var(--text-secondary)] space-y-2">
				<p>Your pack is a valid Minecraft resource pack from the moment you create it. To use it:</p>
				<ol class="list-decimal list-inside space-y-1 ml-2">
					<li>Find your pack folder in the <strong>packs</strong> directory.</li>
					<li>Copy the entire folder to <code class="bg-[var(--bg-tertiary)] px-1 rounded text-sm">.minecraft/resourcepacks/</code></li>
					<li>In Minecraft, go to <strong>Options &gt; Resource Packs</strong> and activate your pack.</li>
					<li>Your custom sounds will replace the defaults!</li>
				</ol>
				<p class="mt-2">You can copy the pack at any time, even with just a few sounds recorded. Any sounds you haven't recorded will use the Minecraft defaults.</p>
			</div>
		</section>

		<!-- Settings -->
		<section>
			<h2 class="text-lg font-semibold mb-3 text-primary">Settings</h2>
			<div class="text-[var(--text-secondary)] space-y-2">
				<ul class="list-disc list-inside space-y-1 ml-2">
					<li><strong>Record Key</strong> — Change the key used to record (default: Spacebar).</li>
					<li><strong>Microphone</strong> — Select your audio input device.</li>
					<li><strong>Auto-play</strong> — Automatically play the original sound when entering each recording step.</li>
					<li><strong>Single Recording Mode</strong> — When enabled, a single recording is duplicated to all variants of a sound event. This ensures your custom sound plays 100% of the time.</li>
					<li><strong>Packs Folder</strong> — Change where pack folders are stored.</li>
				</ul>
			</div>
		</section>

		<!-- FAQ -->
		<section>
			<h2 class="text-lg font-semibold mb-3 text-primary">FAQ</h2>
			<div class="text-[var(--text-secondary)] space-y-4">
				<div>
					<p class="font-medium text-[var(--text-primary)]">How do I delete a pack?</p>
					<p>Navigate to the packs folder and delete the pack folder manually. This prevents accidental deletion of your recordings.</p>
				</div>
				<div>
					<p class="font-medium text-[var(--text-primary)]">What is "Single Recording Mode"?</p>
					<p>Many sound events in Minecraft have multiple variants (e.g., a cow has 4 different "moo" sounds). Normally, you'd record each one individually. With Single Recording Mode, you record once and the file is copied to all variants, so your sound plays every time instead of randomly.</p>
				</div>
				<div>
					<p class="font-medium text-[var(--text-primary)]">What are "LONG" sounds?</p>
					<p>These are music tracks, ambient loops, and other long sounds (typically over 30 seconds). They're flagged so you know recording them will take more time.</p>
				</div>
				<div>
					<p class="font-medium text-[var(--text-primary)]">Can I change the Minecraft version of an existing pack?</p>
					<p>Yes! Go to the pack edit screen and change the version. Sounds that exist in both versions will be kept. Sounds only in the old version will be removed.</p>
				</div>
			</div>
		</section>
	</div>
</div>
