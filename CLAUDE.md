# MVPM — Minecraft Voice Pack Maker

## Project Overview
A portable desktop app that lets users record their own voice/sounds to replace Minecraft Java Edition's default sounds, creating custom resource packs. The pack is always a valid, live resource pack at all times — there is NO export step. Users simply copy the pack folder to `.minecraft/resourcepacks/` whenever they want to use it.

Published to GitHub Releases as **MVPM** with description **"Minecraft Voice Pack Maker"**.
Repository: https://github.com/matmcw/MVPM.git

---

## Technology Stack
- **Desktop Framework**: Tauri 2.x (Rust backend, web frontend)
- **Frontend**: Svelte 5 + SvelteKit + TypeScript
- **State Management**: Svelte 5 runes (`$state`, `$derived`) in `.svelte.ts` store files (not traditional Svelte stores)
- **Styling**: Tailwind CSS 4 (via `@tailwindcss/vite` plugin; no `tailwind.config.js`; theme tokens in `app.css` using `@theme`)
- **Audio Playback**: HTML5 Audio (plays .ogg files from Mojang CDN or local cache)
- **Audio Recording**: Web Audio API + MediaRecorder → WAV
- **Audio Conversion**: Bundled ffmpeg sidecar (WAV → OGG Vorbis)
- **Sound Source**: Mojang's official asset pipeline API (Java Edition only)
- **Packaging**: Tauri bundler → .msi/.exe (manual builds when requested, no CI/CD)
- **Build Tool**: Vite
- **SPA Mode**: SvelteKit static adapter with `fallback: 'index.html'`; SSR and prerender disabled in `+layout.ts`
- **Theme**: Light (default) + dark mode toggle. Simple and modern, NOT Minecraft-inspired.

---

## Critical Design Decisions

### No Export Step
The pack IS a valid Minecraft resource pack from the moment it's created. Each recording is saved directly into the correct `assets/minecraft/sounds/` path inside the pack folder. The user copies the folder to their `.minecraft/resourcepacks/` directory at any time.

### No sounds.json in Resource Pack
Minecraft automatically uses resource pack files that match default file paths. Simply placing an .ogg file at `assets/minecraft/sounds/entity/creeper/fuse.ogg` overrides that default sound. No custom sounds.json is needed in the pack.

### Single Recording Mode (Setting, OFF by default)
When enabled, the user records once per sound event and the recording is **duplicated to all variant filenames** for that event. Example: `entity.cow.ambient` has `idle1.ogg`, `idle2.ogg`, `idle3.ogg` → record once, file is copied to all 3 paths. This ensures the custom sound plays 100% of the time (not 1/3). This approach was chosen over sounds.json `"replace": true` so that users can later turn the setting off and re-record individual variants.

### Fully Portable
The app runs entirely from its own folder. No registry entries, no AppData usage. Can be copied to USB and run anywhere. All data (settings, versions, packs) lives within the app folder.

### Per-Pack Sound Storage
Each pack stores its own copy of original Minecraft sounds. Sounds are NOT shared between packs. This simplifies management even though it uses more disk space.

### No Pack Deletion in App
Users delete pack folders manually via file explorer. This prevents accidental deletion of hours of recordings.

### Audio Quality
Fixed sensible defaults: 44.1kHz, mono, ~128kbps OGG Vorbis. Not user-configurable.

### No Volume Control
Original sounds play at default volume. Users adjust system volume if needed.

### No Keyboard Shortcuts Beyond Record Key
Only the record key (spacebar by default, configurable) is a keyboard shortcut. Everything else is mouse/click. Escape and Enter work for dialogs.

---

## Project Architecture

### Source Structure
```
MVPM/
├── src-tauri/                       # Rust backend
│   ├── src/
│   │   ├── main.rs                  # Binary entry point
│   │   ├── lib.rs                   # Tauri builder, plugin + command registration
│   │   ├── models.rs                # Shared types (Pack, Settings, SoundNode, etc.)
│   │   └── commands/
│   │       ├── mod.rs               # Module declarations
│   │       ├── mojang.rs            # Version manifest, asset index, sound downloads
│   │       ├── packs.rs             # Pack CRUD, metadata, duplication, version change
│   │       ├── recording.rs         # Save WAV, invoke ffmpeg, file I/O
│   │       └── settings.rs          # Read/write settings.json
│   ├── bin/                         # ffmpeg sidecar (gitignored — see Dev Setup)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                             # Svelte 5 frontend
│   ├── app.css                      # Global styles + Tailwind 4 @theme tokens
│   ├── app.html
│   ├── app.d.ts
│   ├── lib/
│   │   ├── index.ts
│   │   ├── assets/
│   │   │   └── favicon.svg
│   │   ├── components/
│   │   │   ├── Breadcrumb.svelte        # Directory navigation breadcrumbs
│   │   │   ├── DownloadProgress.svelte  # Version download progress modal
│   │   │   ├── HelpButton.svelte        # ? icon present on every screen
│   │   │   ├── PackCard.svelte          # Pack list item on home screen
│   │   │   ├── SearchBar.svelte         # Global sound search
│   │   │   ├── ThemeToggle.svelte       # Light/dark mode switch
│   │   │   ├── Tile.svelte             # Individual tile (sound or category)
│   │   │   ├── TileGrid.svelte         # Sound/category tile grid
│   │   │   └── WarningDialog.svelte     # Reusable warning/confirmation dialog
│   │   ├── stores/                  # Svelte 5 rune-based state (.svelte.ts)
│   │   │   ├── pack.svelte.ts       # Current pack state, recording progress
│   │   │   ├── recording.svelte.ts  # Recording workflow state
│   │   │   ├── settings.svelte.ts   # User preferences (persisted)
│   │   │   ├── sounds.svelte.ts     # Sound tree for current version
│   │   │   └── versions.svelte.ts   # Cached version list, download status
│   │   └── utils/
│   │       ├── api.ts               # Tauri command wrappers + shared types
│   │       └── audio.ts             # Playback + recording helpers
│   └── routes/
│       ├── +layout.svelte           # App shell, help button, theme
│       ├── +layout.ts               # SSR/prerender disabled (SPA mode)
│       ├── +page.svelte             # Home: pack list + create button
│       ├── create/
│       │   ├── version/+page.svelte     # Wizard step 1: version select
│       │   ├── details/+page.svelte     # Wizard step 2: name + description
│       │   └── icon/+page.svelte        # Wizard step 3: optional icon
│       ├── pack/[id]/
│       │   ├── +page.svelte         # Pack editor: tile grid
│       │   └── edit/+page.svelte    # Edit pack name/description/icon/version
│       ├── record/+page.svelte      # Recording workflow screen
│       ├── settings/+page.svelte    # Settings page
│       └── help/+page.svelte        # Help/guide page
├── static/
├── package.json
├── svelte.config.js                 # SvelteKit config (static adapter, SPA fallback)
├── vite.config.ts                   # Vite config (Tailwind + SvelteKit plugins)
└── tsconfig.json
```

### Dev Setup Notes
- `src-tauri/bin/` is **gitignored**. Developers must place a static ffmpeg build at `src-tauri/bin/ffmpeg-x86_64-pc-windows-msvc.exe`.
- Tailwind CSS 4 has no config file; theme tokens are defined via `@theme` in `src/app.css`.
- `pack_format` for `pack.mcmeta` is dynamically derived from Mojang's version JSON (downloaded with each version), with a hardcoded fallback table for edge cases.

### Portable Runtime File Structure
```
MVPM/
├── MVPM.exe                       # Main executable
├── ffmpeg.exe                     # Bundled sidecar
├── settings.json                  # User preferences (persisted)
├── versions/                      # Downloaded version sound caches
│   ├── 1.21.4/
│   │   ├── asset_index.json       # Cached asset index
│   │   ├── sounds.json            # Cached sounds.json (for event→file mapping)
│   │   ├── download_status.json   # Tracks complete/incomplete downloads
│   │   └── sounds/                # Original .ogg files (by path)
│   │       └── minecraft/sounds/entity/creeper/fuse.ogg
│   └── ...
├── packs/                         # User's resource packs (location configurable in settings)
│   └── MyVoicePack/
│       ├── pack.mcmeta            # Auto-generated on pack creation
│       ├── pack.png               # Optional user-uploaded icon
│       ├── pack_meta.json         # Internal metadata: version, recorded sounds list
│       └── assets/
│           └── minecraft/
│               └── sounds/        # User's recordings
│                   └── entity/
│                       └── creeper/
│                           └── fuse.ogg
└── version_cache.json             # Cached version manifest for offline fallback
```

---

## Screen-by-Screen Specification

### 1. Home Screen (`/`)
- **Simple list** of existing packs, each showing: name, description, version, progress (e.g., "47/823 sounds recorded")
- **"Create New Pack" button** prominently displayed
- **Pack actions**: Click pack to open pack editor. Pencil icon → edit screen. Duplicate button → prompts for new name (no duplicate pack names allowed)
- **No delete** — users delete pack folders manually via file explorer
- **Guide section**: Link/button to help page

### 2. Pack Creation Wizard (`/create/version` → `/create/details` → `/create/icon`)

**Step 1 — Version** (`/create/version`):
- List all Minecraft versions fetched from Mojang API on app startup
- Falls back to locally cached version list if offline
- **Checkbox to filter releases only** (hides snapshots, betas, alphas)
- When user selects a version and clicks "Next":
  - If version sounds are already downloaded: proceed to step 2
  - If NOT downloaded: popup asks "Would you like to download sounds for this version?"
    - Shows download progress (X of Y files, MB downloaded)
    - Each failed file retries 3 times automatically
    - If still failing: error dialog with **Retry** and **Exit** buttons
    - Exit keeps already-downloaded files, marks version as **incomplete** (cannot create pack with incomplete version)
    - On success: dismiss popup, return to version screen, user clicks "Next" again to proceed

**Step 2 — Details** (`/create/details`):
- Name text field (required, must be unique among existing packs)
- Description text field (required)

**Step 3 — Icon** (`/create/icon`):
- Optional pack.png upload
- **"Skip" button** to skip icon selection
- On completion: create pack folder with `pack.mcmeta` immediately (valid empty resource pack from the start)

### 3. Pack Editor — Tile Grid (`/pack/[id]`)
- **CSS Grid** of tiles with responsive columns
- **Category tiles**: Distinct border/color from sound tiles. **Double-click** to enter (navigate into subdirectory)
- **Sound tiles**: Show filename. **Green background** if recorded, **default** if not recorded
- **Category completion**: Category tile turns **green** when ALL sounds within it (recursively) are recorded
- **Music/long sounds**: Flagged with a **duration indicator** label on the tile (for sounds > 30s like music tracks and records)
- **Breadcrumb navigation**: Shows current path (e.g., `sounds > entity > creeper`). Click any segment to navigate up
- **Selection**:
  - Single-click to select/deselect a tile (highlighted in **blue**)
  - Multi-select supported
  - **Clicking a category selects ALL sounds within it recursively** (including all subdirectories)
- **Search bar**: Global search across ALL sounds for the version (not just current directory). Results shown as a flat list. Clicking a result navigates to its directory with it highlighted. This lets users find specific mob/block sounds easily.
- **"Record Selected" button**:
  - If any selected sounds are already recorded: warning dialog — "You have already recorded some of the selected sounds. Would you like to re-record them? This will permanently overwrite the recordings." Options: **Re-record** / **Skip recorded ones**
  - Opens recording workflow with selected sounds

### 4. Recording Workflow (`/record`)

**Layout:**
- Sound name (large text)
- Progress counter ("Recording 3 of 47")
- Simple waveform visualizer
- Timer with milliseconds
- Color-coded indicator:
  - **Gray** = waiting for user to record
  - **Red** = actively recording
  - **Green** = this sound already has a recording (visible when navigating back to review)

**Auto-play:** Original sound plays automatically when entering each recording step (configurable in settings).

**Controls (visible buttons):**
- Play original sound (replay button)
- Play my recording (playback of saved recording, if one exists)
- Re-record this sound
- Previous sound (arrow ←)
- Next/Skip sound (arrow →)
- Done/Exit recording button
- **"Automatically skip to next unrecorded sound" toggle** (on the recording screen itself, NOT in settings)

**Recording flow:**
1. Original sound auto-plays (if setting enabled)
2. User holds spacebar (configurable key) → recording starts (red indicator, waveform active, timer counting)
3. User releases spacebar → recording stops, WAV saved, ffmpeg converts to OGG, file placed in pack directory
4. If auto-skip toggle **ON**: automatically advance to next **UNRECORDED** sound (skips already-recorded ones)
5. If auto-skip toggle **OFF**: stay on current sound, user manually clicks arrows to navigate

**Toggle behavior:**
- If auto-skip is ON and user manually clicks a prev/next arrow, **the toggle automatically turns OFF** (user is now manually navigating)

**Done button behavior:**
- If any selected sounds are still unrecorded when user clicks Done:
  - Warning: "One or more selected sounds were skipped. Do you want to record them now?"
  - **"Yes"** → jump to first unrecorded sound, auto-skip toggle turns ON, iterate through unrecorded sounds
  - **"Skip"** → return to tile grid (unrecorded sounds remain not-green)
- If all sounds recorded: return directly to tile grid

**Sound order:** Sounds presented in **directory order** (matching the file tree structure).

### 5. Pack Edit Screen (`/pack/[id]/edit`)
- Edit pack **name**, **description**, and **icon** (upload new pack.png)
- **Duplicate pack**: Button that copies the entire pack folder. Prompts for new unique name.
- **Change version**:
  1. Select new version from dropdown
  2. If new version not downloaded, trigger the download flow
  3. Warning: "Changing version will remove sounds that don't exist in the new version and add new sound slots. Recordings for sounds that exist in both versions will be kept. We recommend duplicating the pack first."
  4. On confirm: delete recordings for sounds removed in new version, add new sound entries to metadata, update `pack.mcmeta` with correct `pack_format`

### 6. Settings (`/settings`)
- **Record key binding**: Change from spacebar to another key
- **Microphone selection**: Dropdown of available audio input devices
- **Auto-play original sound**: Toggle (ON by default) — whether original sound auto-plays on each recording step
- **Single recording mode**: Toggle (OFF by default) — when ON, record once per sound event, duplicate file to all variant filenames
- **Packs folder location**: File picker to change where packs directory is located (default: `[app_folder]/packs/`)
- **Theme**: Light / Dark toggle
- Additional sensible settings may be added during development

### 7. Help Page (`/help`)
- **Sections**: Getting Started, Recording Sounds, Using Your Pack (explain copy-to-resourcepacks), Settings, FAQ
- Accessible from **every screen** via a `?` help button in the corner of the app shell

---

## Mojang Asset Pipeline (Sound Sourcing)

### How It Works
1. **Version manifest**: Fetch `https://launchermeta.mojang.com/mc/game/version_manifest.json` → lists ALL Java Edition versions (releases, snapshots, old_alpha, old_beta)
2. **Cache locally**: Save to `version_cache.json` for offline fallback
3. **On app startup**: Try to fetch fresh manifest. If offline, use cached version.
4. **Per version**: Fetch version JSON → contains `assetIndex` URL → download asset index JSON
5. **Parse asset index**: Filter entries starting with `minecraft/sounds/` → build directory tree
6. **Also download sounds.json**: This maps sound events to files (needed for single recording mode grouping and for flagging long sounds)
7. **Download sound files**: `https://resources.download.minecraft.net/{first2chars_of_hash}/{full_hash}` → save to `versions/{version_id}/sounds/` preserving original directory paths
8. **Track download status**: `download_status.json` per version marks complete vs incomplete

### Sound Tree Data Model
```typescript
interface SoundNode {
  name: string;           // e.g., "creeper"
  path: string;           // e.g., "minecraft/sounds/entity/creeper"
  type: 'directory' | 'file';
  children?: SoundNode[]; // subdirectories and files
  hash?: string;          // for files: Mojang asset hash
  size?: number;          // for files: file size in bytes
  isRecorded?: boolean;   // has user recording in current pack
  isLongSound?: boolean;  // flag for music/records (duration > 30s)
}
```

### How Minecraft Sound Override Works
- Resource packs work as file overlays: placing a file at the same path as a default overrides it
- `assets/minecraft/sounds/entity/cow/idle1.ogg` in your pack replaces the default `idle1.ogg`
- If a sound event has 3 variants and you only replace 1, your sound plays 1/3 of the time, defaults play 2/3
- A `sounds.json` with `"replace": true` can force Minecraft to ONLY use specified files — but we chose NOT to use this approach (see Single Recording Mode above)

---

## Audio Pipeline

1. **Record**: Web Audio API + MediaRecorder → capture as WAV (`audio/wav` MIME type)
2. **Save temp**: Write WAV blob to temp file via Tauri filesystem API
3. **Convert**: Tauri Rust command invokes ffmpeg sidecar: `ffmpeg -i temp.wav -c:a libvorbis -q:a 5 -y output.ogg`
4. **Place**: Move `output.ogg` to correct path in pack's `assets/minecraft/sounds/` directory
5. **Cleanup**: Delete temp WAV file
6. **Single mode**: If single recording mode is ON, also copy the .ogg to all other variant filenames for the same sound event

### ffmpeg Bundling
- ffmpeg.exe bundled via Tauri's sidecar system
- Configured in `tauri.conf.json` under `bundle.externalBin`
- Auto-included in builds

---

## pack.mcmeta Format
```json
{
  "pack": {
    "pack_format": <version_specific_number>,
    "description": "<user's pack description>"
  }
}
```
`pack_format` is dynamically derived from Mojang's version JSON (`resource_pack_version` field in the `compliance` section) at download time. A hardcoded fallback table in `mojang.rs` (`fallback_pack_format`) covers cases where the version JSON is unavailable.

---

## Offline Behavior
- App works fully offline with already-downloaded versions
- Version list falls back to `version_cache.json` (cached from last successful online fetch)
- Subtle "offline" indicator shown when network is unavailable
- Only fails when trying to download new or incomplete versions
- All recording, browsing, and pack management work offline

---

## Window Behavior
- **Remember size and position** between sessions (stored in `settings.json`)
- **Minimum window size**: 800×600
- Resizable freely above minimum

---

## Implementation Status
All planned features have been implemented in the initial codebase. The build order below reflects the sequence used and can guide future contributors:

1. **Scaffold**: Tauri 2 + Svelte 5 + SvelteKit + Tailwind CSS 4 + Vite project setup
2. **Rust backend**: All Tauri commands (Mojang API, pack management, file I/O, settings, ffmpeg invocation)
3. **Data stores**: Svelte 5 rune-based stores (`.svelte.ts`) for versions, packs, sounds, recording, settings
4. **Layout + routing**: App shell with help button, theme toggle, all routes (SPA mode via static adapter)
5. **Home screen**: Pack list, create button
6. **Pack creation wizard**: 3-step flow with version download progress
7. **Pack editor (tile grid)**: Full grid with categories, sounds, search, breadcrumbs, selection
8. **Recording workflow**: Complete recording screen with all controls and toggle logic
9. **Settings page**: All settings
10. **Help page**: All sections
11. **Pack edit screen**: Edit details, duplicate, version change
12. **Polish**: Theme implementation (light default + dark), transitions, error handling, edge cases
13. **Packaging config**: Tauri bundler configuration for portable .exe + ffmpeg sidecar

### Development Commands
```
npm install                    # Install frontend dependencies
npx @tauri-apps/cli dev        # Run in development mode (Rust + frontend hot-reload)
npx @tauri-apps/cli build      # Build production executable with ffmpeg sidecar
npm run build                  # Build frontend only
npm run check                  # TypeScript/Svelte type checking
```

---

## Verification / Testing Checklist
1. `npx @tauri-apps/cli dev` launches the app successfully
2. Version list populates from Mojang API (or falls back to cache if offline)
3. Create a new pack: complete 3-step wizard, verify pack folder + pack.mcmeta created on disk
4. Download a version's sounds: progress bar works, all files download, retry logic works
5. Tile grid: navigate categories via double-click, breadcrumbs work, search finds sounds globally
6. Selection: click selects, category click selects all children recursively, blue highlight appears
7. Record a sound: hold spacebar → release → OGG file appears at correct path in pack directory
8. Auto-skip toggle: ON advances to next unrecorded sound, clicking arrow turns toggle OFF
9. Done button: warns about unrecorded sounds, "Yes" loops through them, "Skip" returns to grid
10. Re-record warning appears when selecting already-recorded sounds
11. Single recording mode: one recording duplicated to all variant filenames
12. Pack is valid: copy pack folder to `.minecraft/resourcepacks/` → custom sounds play in Minecraft
13. Settings persist across app restarts (settings.json on disk)
14. Window size/position remembered between sessions
15. Offline mode: disconnect internet → app works with cached/downloaded data
16. Theme toggle switches between light and dark modes
17. Help page accessible from every screen via ? button
18. Pack edit: rename, change description, change icon, duplicate, version change all work
19. `npx @tauri-apps/cli build` produces working portable .exe with ffmpeg sidecar
