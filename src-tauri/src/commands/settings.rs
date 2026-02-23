use crate::models::Settings;
use std::path::PathBuf;

fn get_app_root() -> PathBuf {
	std::env::current_exe()
		.expect("Failed to get executable path")
		.parent()
		.expect("Failed to get executable directory")
		.to_path_buf()
}

fn settings_path() -> PathBuf {
	get_app_root().join("settings.json")
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
	let path = settings_path();
	if path.exists() {
		let content = std::fs::read_to_string(&path)
			.map_err(|e| format!("Failed to read settings: {}", e))?;
		let settings: Settings = serde_json::from_str(&content)
			.map_err(|e| format!("Failed to parse settings: {}", e))?;
		Ok(settings)
	} else {
		Ok(Settings::default())
	}
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
	let path = settings_path();
	let content = serde_json::to_string_pretty(&settings)
		.map_err(|e| format!("Failed to serialize settings: {}", e))?;
	std::fs::write(&path, content)
		.map_err(|e| format!("Failed to write settings: {}", e))?;
	Ok(())
}

pub fn get_packs_folder(settings: &Settings) -> PathBuf {
	match &settings.packs_folder {
		Some(folder) => PathBuf::from(folder),
		None => get_app_root().join("packs"),
	}
}

pub fn get_versions_folder() -> PathBuf {
	get_app_root().join("versions")
}

pub fn get_version_cache_path() -> PathBuf {
	get_app_root().join("version_cache.json")
}
