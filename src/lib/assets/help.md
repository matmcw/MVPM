# Getting Started

MVPM lets you create custom Minecraft resource packs by recording your own sounds to replace the default game sounds.

1. Click **"Create New Pack"** on the home screen.
2. Select the Minecraft version you play on.
3. Give your pack a name and description.
4. Optionally add a pack icon.
5. Start recording sounds!

# Recording Sounds

Navigate the sound categories by clicking folder tiles. Select individual sounds or entire categories using the checkboxes, then press **"Record Selected"**.

You can also drag to select multiple tiles at once.

In the recording screen:

- **Hold the record key** (spacebar by default) to record.
- **Release** to stop recording. The file is saved automatically.
- Use the **Play Original** button to hear the default sound.
- Use the **Play Recording** button to hear your recording.
- Enable **"Auto-skip"** to automatically move to the next unrecorded sound.

# Using Your Pack in Minecraft

Your pack is a valid Minecraft resource pack from the moment you create it. To use it:

1. Find your pack folder in the **packs** directory.
2. Copy the entire folder to `.minecraft/resourcepacks/`
3. In Minecraft, go to **Options > Resource Packs** and activate your pack.
4. Your custom sounds will replace the defaults!

You can copy the pack at any time, even with just a few sounds recorded. Any sounds you haven't recorded will use the Minecraft defaults.

# Settings

- **Record Key** — Change the key used to record (default: Spacebar).
- **Microphone** — Select your audio input device.
- **Auto-play** — Automatically play the original sound when entering each recording step.
- **Single Recording Mode** — When enabled, a single recording is duplicated to all variants of a sound event. This ensures your custom sound plays 100% of the time.
- **Packs Folder** — Change where pack folders are stored.

# FAQ

## What Minecraft versions are supported?

MVPM supports every Java Edition version available in the Mojang version manifest, including releases, snapshots, and betas/alphas.
MVPM does not support Minecraft Bedrock or any other version.

## Does this work offline?

Yes! You only need an internet connection to download sounds for a Minecraft version. Once downloaded, everything else works fully offline.

## Does MVPM modify my Minecraft installation?

No, MVPM only modifies the resource packs in the set output folder. You must copy them into your `.minecraft/resourcepacks/` directory yourself.

Note: It is possible to set the output directory to `.minecraft.resourcepacks/`, but this is not recommended.

## What is "Single Recording Mode"?

Many sounds in Minecraft have multiple variants. With Single Recording Mode, you only record each sound once and the file is duplicated for each varient.

Say, for instance, you are recording the cow's "moo" sound. With this mode OFF, you record each sound seperately resulting in 4 diferent possible sounds being played at random. With this mode ON, 4 sound files will be created, each with the exact same recording.

## What are "LONG" sounds?

Long sounds are music tracks, ambient loops, and other long sounds (typically over 30 seconds). They're flagged so you know recording them will take more time.

## What happens if I record over an existing recording?

The old recording is deleted permanently. There is no way to undo re-recording a sound, so ne careful.

## Can I use pre-recorded audio files instead of recording live?

Yes, but they need to be in OGG Vorbis format (the format used by minecraft). You can manually place audio files into the correct directory inside your pack folder and they'll work just like any other recording.

## Can I change the Minecraft version of an existing pack?

Yes! Go to the pack edit screen and change the version. Sounds that exist in both versions will be kept. Sounds only in the old version will be removed. It is recommended to duplicate your pack before cahnging the version.

## How do I delete a pack?

Click the trash icon on a pack from the home screen, or use the Delete Pack button on the pack edit page. You'll need to type the pack name to confirm. You can also delete the pack from the output folder.

# Legal

MVPM is not affiliated with, endorsed by, or associated with Mojang Studios or Microsoft. Minecraft is a trademark of Mojang Studios.

MVPM is provided as-is with no warranty. See the LICENSE file for details.
