use crate::models::PackMeta;
use std::path::PathBuf;
use tauri_plugin_shell::ShellExt;

fn pack_sounds_dir(pack_id: &str) -> Result<PathBuf, String> {
	let settings = load_settings();
	let packs_folder = crate::commands::settings::get_packs_folder(&settings);
	Ok(packs_folder.join(pack_id).join("assets/minecraft/sounds"))
}

#[tauri::command]
pub async fn save_recording(
	app: tauri::AppHandle,
	pack_id: String,
	sound_path: String,
	wav_data: Vec<u8>,
	single_mode: bool,
	variant_paths: Option<Vec<String>>,
) -> Result<(), String> {
	let sounds_dir = pack_sounds_dir(&pack_id)?;
	std::fs::create_dir_all(&sounds_dir)
		.map_err(|e| format!("Failed to create sounds directory: {}", e))?;

	// Write WAV to temp file
	let temp_dir = std::env::temp_dir();
	let temp_wav = temp_dir.join(format!("mvpm_recording_{}.wav", uuid::Uuid::new_v4()));
	let temp_ogg = temp_dir.join(format!("mvpm_recording_{}.ogg", uuid::Uuid::new_v4()));

	std::fs::write(&temp_wav, &wav_data)
		.map_err(|e| format!("Failed to write temp WAV: {}", e))?;

	// Convert WAV to OGG using ffmpeg sidecar
	let output = app
		.shell()
		.sidecar("ffmpeg")
		.map_err(|e| format!("Failed to create ffmpeg sidecar: {}", e))?
		.args([
			"-i",
			&temp_wav.to_string_lossy(),
			"-c:a",
			"libvorbis",
			"-q:a",
			"5",
			"-ac",
			"1",
			"-ar",
			"44100",
			"-y",
			&temp_ogg.to_string_lossy(),
		])
		.output()
		.await
		.map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

	// Clean up temp WAV
	let _ = std::fs::remove_file(&temp_wav);

	if !output.status.success() {
		let _ = std::fs::remove_file(&temp_ogg);
		return Err(format!(
			"ffmpeg conversion failed: {}",
			String::from_utf8_lossy(&output.stderr)
		));
	}

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

	// Move OGG to destination
	std::fs::copy(&temp_ogg, &dest_path)
		.map_err(|e| format!("Failed to copy recording to pack: {}", e))?;

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
					std::fs::copy(&temp_ogg, &var_dest)
						.map_err(|e| format!("Failed to copy variant: {}", e))?;
					all_recorded_paths.push(variant.clone());
				}
			}
		}
	}

	// Clean up temp OGG
	let _ = std::fs::remove_file(&temp_ogg);

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
