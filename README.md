# MVPM -- Minecraft Voice Pack Maker

A portable desktop app that lets you record your own voice and sounds to replace Minecraft Java Edition's default sounds, creating custom resource packs. Your pack is always a valid, live resource pack -- there is no export step. Just copy the pack folder to `.minecraft/resourcepacks/` and go.

## Features

- Browse every sound in any Minecraft Java Edition version (fetched from Mojang's official asset API)
- Record your voice or any sound to replace individual Minecraft sounds
- Tile grid editor with categories, search, breadcrumb navigation, and multi-select
- In-app pack deletion with type-to-confirm safety prompt
- Single recording mode: record once per sound event and duplicate to all variants
- Light and dark themes
- Fully portable -- runs from its own folder with no registry or AppData usage
- Works offline with previously downloaded version data

## Technology Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2.x (Rust backend, web frontend) |
| Frontend | Svelte 5 + SvelteKit + TypeScript |
| Styling | Tailwind CSS 4 |
| Audio recording | Web Audio API + MediaRecorder (WAV capture) |
| Audio conversion | Bundled ffmpeg sidecar (WAV to OGG Vorbis) |
| Sound source | Mojang's official Java Edition asset pipeline API |
| Build tool | Vite |

## Prerequisites

Before you can build or run MVPM, make sure you have:

- **Rust** and **Cargo** (install via [rustup](https://rustup.rs/))
- **Node.js** (LTS recommended) and **npm**
- **Tauri CLI** -- installed globally (`npm install -g @tauri-apps/cli`) or invoked via `npx`
- **ffmpeg binary** -- a static Windows build of ffmpeg placed at:
  ```
  src-tauri/bin/ffmpeg-x86_64-pc-windows-msvc.exe
  ```
  You can download a static build from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) or [BtbN's GitHub releases](https://github.com/BtbN/FFmpeg-Builds/releases). Only `ffmpeg.exe` is needed (not ffprobe or ffplay). Rename it to match the path above.

> The `src-tauri/bin/` directory is gitignored because the ffmpeg binary is too large for source control. Every developer must place it manually.

## Getting Started

```bash
# 1. Clone the repository
git clone https://github.com/matmcw/MVPM.git
cd MVPM

# 2. Install Node dependencies
npm install

# 3. Place ffmpeg (see Prerequisites above)
#    src-tauri/bin/ffmpeg-x86_64-pc-windows-msvc.exe

# 4. Run in development mode
npx @tauri-apps/cli dev
```

The app will open a window. On first launch it fetches the Minecraft version manifest from Mojang and lets you create your first voice pack.

## Development Commands

| Command | Description |
|---------|-------------|
| `npm install` | Install frontend dependencies |
| `npx @tauri-apps/cli dev` | Run the full app in development mode (Rust + frontend hot-reload) |
| `npx @tauri-apps/cli build` | Build a production executable with bundled ffmpeg sidecar |
| `npm run build` | Build the frontend only (output in `build/`) |
| `npm run check` | Run the Svelte/TypeScript type checker |
| `npm run check:watch` | Run the type checker in watch mode |

## Project Structure

```
MVPM/
├── src-tauri/                       # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs                  # Binary entry point
│   │   ├── lib.rs                   # Tauri builder, plugin + command registration
│   │   ├── models.rs                # Shared types (Pack, Settings, SoundNode, etc.)
│   │   └── commands/
│   │       ├── mod.rs               # Module declarations
│   │       ├── mojang.rs            # Version manifest, asset index, sound downloads
│   │       ├── packs.rs             # Pack CRUD, metadata, duplication, version change
│   │       ├── recording.rs         # Save WAV, invoke ffmpeg sidecar, file I/O
│   │       └── settings.rs          # Read/write settings.json
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                             # Svelte 5 frontend
│   ├── app.css                      # Global styles + Tailwind 4 theme tokens
│   ├── app.html                     # HTML shell
│   ├── lib/
│   │   ├── assets/
│   │   │   ├── favicon.svg
│   │   │   └── help.md              # Help page content (parsed at runtime)
│   │   ├── components/              # Reusable UI components
│   │   │   ├── Breadcrumb.svelte
│   │   │   ├── DownloadProgress.svelte
│   │   │   ├── HelpButton.svelte
│   │   │   ├── PackCard.svelte
│   │   │   ├── SearchBar.svelte
│   │   │   ├── ThemeToggle.svelte
│   │   │   ├── Tile.svelte
│   │   │   ├── TileGrid.svelte
│   │   │   └── WarningDialog.svelte
│   │   ├── stores/                  # Svelte 5 rune-based state (.svelte.ts)
│   │   │   ├── createWizard.svelte.ts
│   │   │   ├── pack.svelte.ts
│   │   │   ├── recording.svelte.ts
│   │   │   ├── settings.svelte.ts
│   │   │   ├── sounds.svelte.ts
│   │   │   └── versions.svelte.ts
│   │   └── utils/
│   │       ├── api.ts               # Tauri invoke wrappers + shared types
│   │       └── audio.ts             # Playback + recording helpers
│   └── routes/
│       ├── +layout.svelte           # App shell (help button, theme toggle)
│       ├── +layout.ts               # Disables SSR/prerender for SPA mode
│       ├── +page.svelte             # Home: pack list + create button
│       ├── create/
│       │   ├── version/+page.svelte # Wizard step 1: pick a Minecraft version
│       │   ├── details/+page.svelte # Wizard step 2: name + description
│       │   └── icon/+page.svelte    # Wizard step 3: optional pack icon
│       ├── pack/[id]/
│       │   ├── +page.svelte         # Pack editor: tile grid
│       │   └── edit/+page.svelte    # Edit pack name/description/icon/version
│       ├── record/+page.svelte      # Recording workflow screen
│       ├── settings/+page.svelte    # App settings
│       └── help/+page.svelte        # User guide
├── package.json
├── svelte.config.js                 # SvelteKit config (static adapter, SPA fallback)
├── vite.config.ts                   # Vite config (Tailwind + SvelteKit plugins)
├── tsconfig.json
└── CLAUDE.md                        # Project specification and architecture reference
```

## Architecture Notes

- **Rust backend** handles all file I/O, Mojang API requests, pack management, and ffmpeg invocation via Tauri commands. The frontend calls these through `@tauri-apps/api`.
- **Frontend state** uses Svelte 5 runes (`$state`, `$derived`) in `.svelte.ts` store files rather than traditional Svelte stores.
- **SvelteKit** is configured with the static adapter in SPA/fallback mode (`+layout.ts` disables SSR and prerendering).
- **Tailwind CSS 4** is integrated via the `@tailwindcss/vite` plugin. There is no `tailwind.config.js`; theme tokens are defined with `@theme` in `app.css`.
- **pack_format** for `pack.mcmeta` is dynamically derived from Mojang's version JSON at download time, with a hardcoded fallback table for edge cases.
- **ffmpeg** is configured as a Tauri sidecar in `tauri.conf.json` under `bundle.externalBin`.

## How It Works

1. **Create a pack** -- pick a Minecraft version, enter a name and description, optionally upload an icon. The app downloads all sounds for that version from Mojang's CDN.
2. **Browse sounds** -- navigate the tile grid by category (entity, block, music, etc.). Search globally for any sound.
3. **Record** -- select sounds and enter the recording workflow. Hold the record key (spacebar by default) to record, release to stop. The WAV is automatically converted to OGG Vorbis and placed at the correct path in the pack.
4. **Use the pack** -- copy your pack folder to `.minecraft/resourcepacks/`, enable it in Minecraft, and hear your recordings in-game.

## Disclaimer

MVPM is not affiliated with, endorsed by, or associated with Mojang Studios or Microsoft. Minecraft is a trademark of Mojang Studios. All Minecraft assets are downloaded directly from Mojang's public CDN to the user's own machine and are not redistributed.

This application bundles [ffmpeg](https://ffmpeg.org/) for audio conversion. ffmpeg is licensed under the [LGPL 2.1](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html) (or GPL, depending on build configuration). The ffmpeg source code is available at [https://ffmpeg.org/download.html](https://ffmpeg.org/download.html). MVPM does not modify ffmpeg; it is included as a standalone binary sidecar.

## License

[MIT](LICENSE)
