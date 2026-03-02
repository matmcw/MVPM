use crate::commands::mojang::get_pack_format_for_version;
use crate::commands::settings::{get_packs_folder, get_settings as load_settings_sync};
use crate::models::*;
use std::path::{Path, PathBuf};

fn packs_dir() -> Result<PathBuf, String> {
	let settings = tokio::task::block_in_place(|| {
		let rt = tokio::runtime::Handle::current();
		rt.block_on(load_settings_sync())
	})?;
	let dir = get_packs_folder(&settings);
	if !dir.exists() {
		std::fs::create_dir_all(&dir)
			.map_err(|e| format!("Failed to create packs directory: {}", e))?;
	}
	Ok(dir)
}

fn pack_dir(pack_id: &str) -> Result<PathBuf, String> {
	Ok(packs_dir()?.join(pack_id))
}

fn load_pack_meta(pack_id: &str) -> Result<PackMeta, String> {
	let meta_path = pack_dir(pack_id)?.join("pack_meta.json");
	let content = std::fs::read_to_string(&meta_path)
		.map_err(|e| format!("Failed to read pack metadata: {}", e))?;
	let mut meta: PackMeta = serde_json::from_str(&content)
		.map_err(|e| format!("Failed to parse pack metadata: {}", e))?;
	populate_icon_path(&mut meta);
	meta.recorded_sounds = scan_recorded_sounds(pack_id);
	meta.total_sounds = count_version_sounds(&meta.version_id);
	Ok(meta)
}

fn count_version_sounds(version_id: &str) -> u32 {
	let ver_dir = crate::commands::mojang::version_dir(version_id);
	let ai_path = ver_dir.join("asset_index.json");
	if let Ok(content) = std::fs::read_to_string(&ai_path) {
		if let Ok(index) = serde_json::from_str::<serde_json::Value>(&content) {
			if let Some(objects) = index.get("objects").and_then(|o| o.as_object()) {
				return objects
					.keys()
					.filter(|k| k.starts_with("minecraft/sounds/") && k.ends_with(".ogg"))
					.count() as u32;
			}
		}
	}
	0
}

fn scan_recorded_sounds(pack_id: &str) -> Vec<String> {
	let sounds_dir = match pack_dir(pack_id) {
		Ok(dir) => dir.join("assets").join("minecraft").join("sounds"),
		Err(_) => return vec![],
	};
	if !sounds_dir.exists() {
		return vec![];
	}
	let mut recorded = Vec::new();
	scan_dir_recursive(&sounds_dir, &sounds_dir, &mut recorded);
	recorded
}

fn scan_dir_recursive(base: &Path, current: &Path, results: &mut Vec<String>) {
	if let Ok(entries) = std::fs::read_dir(current) {
		for entry in entries.flatten() {
			let path = entry.path();
			if path.is_dir() {
				scan_dir_recursive(base, &path, results);
			} else if path.extension().map_or(false, |ext| ext == "ogg") {
				if let Ok(relative) = path.strip_prefix(base) {
					let logical = format!(
						"minecraft/sounds/{}",
						relative.to_string_lossy().replace('\\', "/")
					);
					results.push(logical);
				}
			}
		}
	}
}

fn populate_icon_path(meta: &mut PackMeta) {
	if meta.has_icon {
		if let Ok(dir) = pack_dir(&meta.id) {
			let icon = dir.join("pack.png");
			if icon.exists() {
				meta.icon_path = Some(icon.to_string_lossy().to_string());
			}
		}
	}
}

fn save_pack_meta(meta: &PackMeta) -> Result<(), String> {
	let meta_path = pack_dir(&meta.id)?.join("pack_meta.json");
	let content = serde_json::to_string_pretty(meta)
		.map_err(|e| format!("Failed to serialize pack metadata: {}", e))?;
	std::fs::write(&meta_path, content)
		.map_err(|e| format!("Failed to write pack metadata: {}", e))?;
	Ok(())
}

fn write_pack_mcmeta(pack_id: &str, pack_format: u32, description: &str) -> Result<(), String> {
	let mcmeta = PackMcmeta {
		pack: PackMcmetaInner {
			pack_format,
			description: description.to_string(),
		},
	};
	let content = serde_json::to_string_pretty(&mcmeta)
		.map_err(|e| format!("Failed to serialize pack.mcmeta: {}", e))?;
	let mcmeta_path = pack_dir(pack_id)?.join("pack.mcmeta");
	std::fs::write(&mcmeta_path, content)
		.map_err(|e| format!("Failed to write pack.mcmeta: {}", e))?;
	Ok(())
}

#[tauri::command]
pub async fn list_packs() -> Result<Vec<PackMeta>, String> {
	let dir = packs_dir()?;
	if !dir.exists() {
		return Ok(vec![]);
	}

	let mut packs = vec![];
	let entries = std::fs::read_dir(&dir)
		.map_err(|e| format!("Failed to read packs directory: {}", e))?;

	for entry in entries {
		let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
		let path = entry.path();
		if path.is_dir() {
			let meta_path = path.join("pack_meta.json");
			if meta_path.exists() {
				if let Ok(content) = std::fs::read_to_string(&meta_path) {
					if let Ok(mut meta) = serde_json::from_str::<PackMeta>(&content) {
						populate_icon_path(&mut meta);
						meta.recorded_sounds = scan_recorded_sounds(&meta.id);
						meta.total_sounds = count_version_sounds(&meta.version_id);
						packs.push(meta);
					}
				}
			}
		}
	}

	packs.sort_by(|a, b| a.name.cmp(&b.name));
	Ok(packs)
}

fn validate_pack_name(name: &str) -> Result<(), String> {
	let trimmed = name.trim();
	if trimmed.is_empty() {
		return Err("Pack name cannot be empty.".to_string());
	}
	let invalid_chars = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
	for ch in invalid_chars {
		if trimmed.contains(ch) {
			return Err(format!("Pack name cannot contain '{}'.", ch));
		}
	}
	let reserved = [
		"CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5",
		"COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4",
		"LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
	];
	let upper = trimmed.to_uppercase();
	if reserved
		.iter()
		.any(|r| upper == *r || upper.starts_with(&format!("{}.", r)))
	{
		return Err("This name is reserved by Windows.".to_string());
	}
	if trimmed.ends_with('.') || trimmed.ends_with(' ') {
		return Err("Pack name cannot end with a period or space.".to_string());
	}
	Ok(())
}

#[tauri::command]
pub async fn create_pack(
	name: String,
	description: String,
	version_id: String,
	icon_path: Option<String>,
) -> Result<PackMeta, String> {
	validate_pack_name(&name)?;
	let trimmed_name = name.trim().to_string();

	// Validate unique name
	let existing = list_packs().await?;
	if existing
		.iter()
		.any(|p| p.name.eq_ignore_ascii_case(&trimmed_name))
	{
		return Err("A pack with this name already exists.".to_string());
	}

	let pack_format = get_pack_format_for_version(&version_id)?;
	let id = trimmed_name.clone();
	let dir = packs_dir()?.join(&id);

	// Create directory structure
	std::fs::create_dir_all(dir.join("assets/minecraft/sounds"))
		.map_err(|e| format!("Failed to create pack directory: {}", e))?;

	// Copy icon if provided
	let has_icon = if let Some(ref src) = icon_path {
		let src_path = std::path::Path::new(src);
		if src_path.exists() {
			std::fs::copy(src_path, dir.join("pack.png"))
				.map_err(|e| format!("Failed to copy icon: {}", e))?;
			true
		} else {
			false
		}
	} else {
		false
	};

	// Create pack.mcmeta
	let mcmeta = PackMcmeta {
		pack: PackMcmetaInner {
			pack_format,
			description: description.clone(),
		},
	};
	let mcmeta_content = serde_json::to_string_pretty(&mcmeta)
		.map_err(|e| format!("Failed to serialize pack.mcmeta: {}", e))?;
	std::fs::write(dir.join("pack.mcmeta"), mcmeta_content)
		.map_err(|e| format!("Failed to write pack.mcmeta: {}", e))?;

	// Create pack_meta.json
	let mut meta = PackMeta {
		id: id.clone(),
		name: trimmed_name,
		description,
		version_id: version_id.clone(),
		pack_format,
		has_icon,
		icon_path: None,
		recorded_sounds: vec![],
		created_at: chrono::Utc::now().to_rfc3339(),
		total_sounds: count_version_sounds(&version_id),
	};

	let meta_content = serde_json::to_string_pretty(&meta)
		.map_err(|e| format!("Failed to serialize pack metadata: {}", e))?;
	std::fs::write(dir.join("pack_meta.json"), meta_content)
		.map_err(|e| format!("Failed to write pack metadata: {}", e))?;

	populate_icon_path(&mut meta);
	Ok(meta)
}

#[tauri::command]
pub async fn get_pack(pack_id: String) -> Result<PackMeta, String> {
	load_pack_meta(&pack_id)
}

#[tauri::command]
pub async fn update_pack(
	pack_id: String,
	name: Option<String>,
	description: Option<String>,
	icon_path: Option<String>,
) -> Result<PackMeta, String> {
	let mut meta = load_pack_meta(&pack_id)?;
	let mut current_id = pack_id.clone();

	if let Some(ref new_name) = name {
		validate_pack_name(new_name)?;
		let trimmed = new_name.trim().to_string();
		// Validate unique name (excluding this pack)
		let existing = list_packs().await?;
		if existing
			.iter()
			.any(|p| p.id != pack_id && p.name.eq_ignore_ascii_case(&trimmed))
		{
			return Err("A pack with this name already exists.".to_string());
		}
		// Rename folder if name changed
		if trimmed != pack_id {
			let old_dir = pack_dir(&pack_id)?;
			let new_dir = packs_dir()?.join(&trimmed);
			std::fs::rename(&old_dir, &new_dir)
				.map_err(|e| format!("Failed to rename pack folder: {}", e))?;
			current_id = trimmed.clone();
			meta.id = trimmed.clone();
		}
		meta.name = trimmed;
	}

	if let Some(ref new_desc) = description {
		meta.description = new_desc.clone();
		write_pack_mcmeta(&current_id, meta.pack_format, new_desc)?;
	}

	if let Some(ref src) = icon_path {
		let src_path = std::path::Path::new(src);
		if src_path.exists() {
			let dest = pack_dir(&current_id)?.join("pack.png");
			std::fs::copy(src_path, &dest)
				.map_err(|e| format!("Failed to copy icon: {}", e))?;
			meta.has_icon = true;
			populate_icon_path(&mut meta);
		}
	}

	save_pack_meta(&meta)?;
	Ok(meta)
}

#[tauri::command]
pub async fn duplicate_pack(pack_id: String, new_name: String) -> Result<PackMeta, String> {
	validate_pack_name(&new_name)?;
	let trimmed_name = new_name.trim().to_string();

	// Validate unique name
	let existing = list_packs().await?;
	if existing
		.iter()
		.any(|p| p.name.eq_ignore_ascii_case(&trimmed_name))
	{
		return Err("A pack with this name already exists.".to_string());
	}

	let source_dir = pack_dir(&pack_id)?;
	let new_id = trimmed_name.clone();
	let dest_dir = packs_dir()?.join(&new_id);

	// Deep copy directory
	copy_dir_recursive(&source_dir, &dest_dir)?;

	// Update metadata
	let mut meta = load_pack_meta(&new_id)?;
	meta.id = new_id.clone();
	meta.name = trimmed_name;
	meta.created_at = chrono::Utc::now().to_rfc3339();
	save_pack_meta(&meta)?;

	Ok(meta)
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
	std::fs::create_dir_all(dst)
		.map_err(|e| format!("Failed to create directory: {}", e))?;

	for entry in std::fs::read_dir(src)
		.map_err(|e| format!("Failed to read directory: {}", e))?
	{
		let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
		let src_path = entry.path();
		let dst_path = dst.join(entry.file_name());

		if src_path.is_dir() {
			copy_dir_recursive(&src_path, &dst_path)?;
		} else {
			std::fs::copy(&src_path, &dst_path)
				.map_err(|e| format!("Failed to copy file: {}", e))?;
		}
	}
	Ok(())
}

#[tauri::command]
pub async fn change_pack_version(
	pack_id: String,
	new_version_id: String,
) -> Result<PackMeta, String> {
	let mut meta = load_pack_meta(&pack_id)?;
	let new_pack_format = get_pack_format_for_version(&new_version_id)?;

	// Get sound lists for old and new versions
	let old_tree = crate::commands::mojang::get_sound_tree(meta.version_id.clone(), None).await?;
	let new_tree = crate::commands::mojang::get_sound_tree(new_version_id.clone(), None).await?;

	let old_sounds = collect_all_file_paths(&old_tree);
	let new_sounds = collect_all_file_paths(&new_tree);

	// Find sounds to remove (in old but not in new)
	let removed: Vec<String> = old_sounds
		.iter()
		.filter(|s| !new_sounds.contains(s))
		.cloned()
		.collect();

	// Delete recordings for removed sounds
	let dir = pack_dir(&pack_id)?;
	for sound_path in &removed {
		let rel = sound_path
			.strip_prefix("minecraft/sounds/")
			.unwrap_or(sound_path);
		let file = dir.join("assets/minecraft/sounds").join(rel);
		if file.exists() {
			let _ = std::fs::remove_file(&file);
		}
	}

	meta.version_id = new_version_id;
	meta.pack_format = new_pack_format;

	// Update pack.mcmeta
	write_pack_mcmeta(&pack_id, new_pack_format, &meta.description)?;
	save_pack_meta(&meta)?;

	Ok(meta)
}

fn collect_all_file_paths(nodes: &[SoundNode]) -> Vec<String> {
	let mut paths = vec![];
	for node in nodes {
		if node.node_type == "file" {
			paths.push(node.path.clone());
		}
		if let Some(ref children) = node.children {
			paths.extend(collect_all_file_paths(children));
		}
	}
	paths
}

#[tauri::command]
pub async fn delete_pack(pack_id: String) -> Result<(), String> {
	let dir = pack_dir(&pack_id)?;
	if !dir.exists() {
		return Err("Pack not found.".to_string());
	}
	std::fs::remove_dir_all(&dir)
		.map_err(|e| format!("Failed to delete pack: {}", e))?;
	Ok(())
}

#[tauri::command]
pub async fn get_recorded_sounds(pack_id: String) -> Result<Vec<String>, String> {
	get_recorded_sounds_internal(&pack_id)
}

pub fn get_recorded_sounds_internal(pack_id: &str) -> Result<Vec<String>, String> {
	let meta = load_pack_meta(pack_id)?;
	Ok(meta.recorded_sounds)
}

#[tauri::command]
pub async fn get_pack_sound_path(
	pack_id: String,
	sound_path: String,
) -> Result<Option<String>, String> {
	let rel = sound_path
		.strip_prefix("minecraft/sounds/")
		.unwrap_or(&sound_path);
	let file = pack_dir(&pack_id)?.join("assets/minecraft/sounds").join(rel);

	if file.exists() {
		Ok(Some(file.to_string_lossy().to_string()))
	} else {
		Ok(None)
	}
}

#[tauri::command]
pub async fn open_pack_folder(pack_id: String) -> Result<(), String> {
	let dir = pack_dir(&pack_id)?;
	if !dir.exists() {
		return Err("Pack folder not found.".to_string());
	}
	std::process::Command::new("explorer")
		.arg(&dir)
		.spawn()
		.map_err(|e| format!("Failed to open folder: {}", e))?;
	Ok(())
}
