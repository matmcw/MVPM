use crate::models::PackMeta;
use std::io::Cursor;
use std::num::{NonZeroU32, NonZeroU8};
use std::path::PathBuf;
use vorbis_rs::{VorbisBitrateManagementStrategy, VorbisEncoderBuilder};

fn pack_sounds_dir(pack_id: &str) -> Result<PathBuf, String> {
	let settings = load_settings();
	let packs_folder = crate::commands::settings::get_packs_folder(&settings);
	Ok(packs_folder.join(pack_id).join("assets/minecraft/sounds"))
}

fn wav_to_ogg(wav_data: &[u8]) -> Result<Vec<u8>, String> {
	let reader = hound::WavReader::new(Cursor::new(wav_data))
		.map_err(|e| format!("Failed to read WAV data: {}", e))?;
	let spec = reader.spec();

	let samples_i16: Vec<i16> = reader
		.into_samples::<i16>()
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| format!("Failed to decode WAV samples: {}", e))?;

	// Convert to mono f32 samples
	let channels = spec.channels as usize;
	let mono_f32: Vec<f32> = if channels == 1 {
		samples_i16.iter().map(|&s| s as f32 / 32768.0).collect()
	} else {
		samples_i16
			.chunks(channels)
			.map(|frame| {
				let sum: f32 = frame.iter().map(|&s| s as f32 / 32768.0).sum();
				sum / channels as f32
			})
			.collect()
	};

	let sample_rate = NonZeroU32::new(44100).unwrap();
	let mono_channel = NonZeroU8::new(1).unwrap();
	let mut ogg_out: Vec<u8> = Vec::new();

	let mut encoder = VorbisEncoderBuilder::new_with_serial(
		sample_rate,
		mono_channel,
		&mut ogg_out,
		0,
	)
	.bitrate_management_strategy(VorbisBitrateManagementStrategy::QualityVbr {
		target_quality: 0.5,
	})
	.build()
	.map_err(|e| format!("Failed to create Vorbis encoder: {}", e))?;

	// Encode in blocks of 4096 samples
	for chunk in mono_f32.chunks(4096) {
		encoder
			.encode_audio_block([chunk])
			.map_err(|e| format!("Failed to encode audio block: {}", e))?;
	}

	encoder
		.finish()
		.map_err(|e| format!("Failed to finish encoding: {}", e))?;

	Ok(ogg_out)
}

#[tauri::command]
pub async fn save_recording(
	pack_id: String,
	sound_path: String,
	wav_data: Vec<u8>,
	single_mode: bool,
	variant_paths: Option<Vec<String>>,
) -> Result<(), String> {
	let sounds_dir = pack_sounds_dir(&pack_id)?;
	std::fs::create_dir_all(&sounds_dir)
		.map_err(|e| format!("Failed to create sounds directory: {}", e))?;

	// Convert WAV to OGG Vorbis using native Rust encoding
	let ogg_data = wav_to_ogg(&wav_data)?;

	// Determine destination path
	let rel_path = sound_path
		.strip_prefix("minecraft/sounds/")
		.unwrap_or(&sound_path);
	let dest_path = sounds_dir.join(rel_path);

	// Create parent directories
	if let Some(parent) = dest_path.parent() {
		std::fs::create_dir_all(parent)
			.map_err(|e| format!("Failed to create parent directory: {}", e))?;
	}

	// Write OGG to destination
	std::fs::write(&dest_path, &ogg_data)
		.map_err(|e| format!("Failed to write recording to pack: {}", e))?;

	// If single recording mode, duplicate to all variant paths
	let mut all_recorded_paths = vec![sound_path.clone()];
	if single_mode {
		if let Some(ref variants) = variant_paths {
			for variant in variants {
				if variant != &sound_path {
					let var_rel = variant
						.strip_prefix("minecraft/sounds/")
						.unwrap_or(variant);
					let var_dest = sounds_dir.join(var_rel);
					if let Some(parent) = var_dest.parent() {
						let _ = std::fs::create_dir_all(parent);
					}
					std::fs::write(&var_dest, &ogg_data)
						.map_err(|e| format!("Failed to write variant: {}", e))?;
					all_recorded_paths.push(variant.clone());
				}
			}
		}
	}

	// Update pack metadata
	update_recorded_sounds(&pack_id, &all_recorded_paths)?;

	Ok(())
}

fn update_recorded_sounds(pack_id: &str, new_paths: &[String]) -> Result<(), String> {
	let settings = load_settings();
	let packs_folder = crate::commands::settings::get_packs_folder(&settings);
	let meta_path = packs_folder.join(pack_id).join("pack_meta.json");

	let content = std::fs::read_to_string(&meta_path)
		.map_err(|e| format!("Failed to read pack metadata: {}", e))?;
	let mut meta: PackMeta = serde_json::from_str(&content)
		.map_err(|e| format!("Failed to parse pack metadata: {}", e))?;

	for path in new_paths {
		if !meta.recorded_sounds.contains(path) {
			meta.recorded_sounds.push(path.clone());
		}
	}

	let json = serde_json::to_string_pretty(&meta)
		.map_err(|e| format!("Failed to serialize metadata: {}", e))?;
	std::fs::write(&meta_path, json)
		.map_err(|e| format!("Failed to write metadata: {}", e))?;

	Ok(())
}

fn load_settings() -> crate::models::Settings {
	let app_root = std::env::current_exe()
		.ok()
		.and_then(|p| p.parent().map(|p| p.to_path_buf()))
		.unwrap_or_default();
	std::fs::read_to_string(app_root.join("settings.json"))
		.ok()
		.and_then(|c| serde_json::from_str(&c).ok())
		.unwrap_or_default()
}

#[tauri::command]
pub async fn delete_recording(pack_id: String, sound_path: String) -> Result<(), String> {
	let sounds_dir = pack_sounds_dir(&pack_id)?;
	let rel_path = sound_path
		.strip_prefix("minecraft/sounds/")
		.unwrap_or(&sound_path);
	let file_path = sounds_dir.join(rel_path);

	if file_path.exists() {
		std::fs::remove_file(&file_path)
			.map_err(|e| format!("Failed to delete recording: {}", e))?;
	}

	// Update metadata
	let settings = load_settings();
	let packs_folder = crate::commands::settings::get_packs_folder(&settings);
	let meta_path = packs_folder.join(&pack_id).join("pack_meta.json");

	if let Ok(content) = std::fs::read_to_string(&meta_path) {
		if let Ok(mut meta) = serde_json::from_str::<PackMeta>(&content) {
			meta.recorded_sounds.retain(|s| s != &sound_path);
			if let Ok(json) = serde_json::to_string_pretty(&meta) {
				let _ = std::fs::write(&meta_path, json);
			}
		}
	}

	Ok(())
}
