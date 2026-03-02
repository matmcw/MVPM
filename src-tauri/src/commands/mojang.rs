use crate::commands::settings::{get_version_cache_path, get_versions_folder};
use crate::models::*;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Emitter;

const VERSION_MANIFEST_URL: &str =
	"https://launchermeta.mojang.com/mc/game/version_manifest.json";
const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

pub fn version_dir(version_id: &str) -> PathBuf {
	get_versions_folder().join(version_id)
}

#[tauri::command]
pub async fn fetch_version_manifest() -> Result<Vec<VersionEntry>, String> {
	let cache_path = get_version_cache_path();

	match reqwest::get(VERSION_MANIFEST_URL).await {
		Ok(response) => {
			let manifest: VersionManifest = response
				.json()
				.await
				.map_err(|e| format!("Failed to parse version manifest: {}", e))?;

			// Cache for offline use
			if let Ok(json) = serde_json::to_string_pretty(&manifest) {
				let _ = std::fs::write(&cache_path, json);
			}

			Ok(manifest.versions)
		}
		Err(_) => {
			// Fall back to cached manifest
			if cache_path.exists() {
				let content = std::fs::read_to_string(&cache_path)
					.map_err(|e| format!("Failed to read cached manifest: {}", e))?;
				let manifest: VersionManifest = serde_json::from_str(&content)
					.map_err(|e| format!("Failed to parse cached manifest: {}", e))?;
				Ok(manifest.versions)
			} else {
				Err("No internet connection and no cached version list available.".to_string())
			}
		}
	}
}

#[tauri::command]
pub async fn is_version_downloaded(version_id: String) -> Result<bool, String> {
	let status_path = version_dir(&version_id).join("download_status.json");
	if !status_path.exists() {
		return Ok(false);
	}

	let content = std::fs::read_to_string(&status_path)
		.map_err(|e| format!("Failed to read download status: {}", e))?;
	let status: DownloadStatus = serde_json::from_str(&content)
		.map_err(|e| format!("Failed to parse download status: {}", e))?;

	Ok(status.complete)
}

#[tauri::command]
pub async fn get_download_status(version_id: String) -> Result<DownloadStatus, String> {
	let status_path = version_dir(&version_id).join("download_status.json");
	if !status_path.exists() {
		return Ok(DownloadStatus {
			total_files: 0,
			downloaded_files: 0,
			total_bytes: 0,
			downloaded_bytes: 0,
			complete: false,
			failed_files: vec![],
		});
	}

	let content = std::fs::read_to_string(&status_path)
		.map_err(|e| format!("Failed to read download status: {}", e))?;
	let status: DownloadStatus = serde_json::from_str(&content)
		.map_err(|e| format!("Failed to parse download status: {}", e))?;
	Ok(status)
}

async fn fetch_version_json(version_url: &str) -> Result<VersionJson, String> {
	let response = reqwest::get(version_url)
		.await
		.map_err(|e| format!("Failed to fetch version JSON: {}", e))?;
	let version_json: VersionJson = response
		.json()
		.await
		.map_err(|e| format!("Failed to parse version JSON: {}", e))?;
	Ok(version_json)
}

async fn fetch_asset_index(url: &str) -> Result<AssetIndex, String> {
	let response = reqwest::get(url)
		.await
		.map_err(|e| format!("Failed to fetch asset index: {}", e))?;
	let asset_index: AssetIndex = response
		.json()
		.await
		.map_err(|e| format!("Failed to parse asset index: {}", e))?;
	Ok(asset_index)
}

#[tauri::command]
pub async fn download_version_sounds(
	app: tauri::AppHandle,
	version_id: String,
) -> Result<(), String> {
	let ver_dir = version_dir(&version_id);
	std::fs::create_dir_all(&ver_dir)
		.map_err(|e| format!("Failed to create version directory: {}", e))?;

	// Fetch version manifest to get the URL for this version
	let manifest_versions = fetch_version_manifest().await?;
	let version_entry = manifest_versions
		.iter()
		.find(|v| v.id == version_id)
		.ok_or_else(|| format!("Version {} not found in manifest", version_id))?
		.clone();

	// Fetch version JSON
	let version_json = fetch_version_json(&version_entry.url).await?;

	// Save pack_format info
	if let Some(pv) = &version_json.pack_version {
		let pv_json = serde_json::to_string_pretty(pv)
			.map_err(|e| format!("Failed to serialize pack version: {}", e))?;
		std::fs::write(ver_dir.join("pack_version.json"), pv_json)
			.map_err(|e| format!("Failed to write pack version: {}", e))?;
	}

	// Fetch asset index
	let asset_index = fetch_asset_index(&version_json.asset_index.url).await?;

	// Save asset index
	let ai_json = serde_json::to_string_pretty(&asset_index)
		.map_err(|e| format!("Failed to serialize asset index: {}", e))?;
	std::fs::write(ver_dir.join("asset_index.json"), &ai_json)
		.map_err(|e| format!("Failed to write asset index: {}", e))?;

	// Filter sound files and sounds.json
	let sound_files: Vec<(String, AssetObject)> = asset_index
		.objects
		.iter()
		.filter(|(key, _)| key.starts_with("minecraft/sounds/"))
		.map(|(key, obj)| (key.clone(), obj.clone()))
		.collect();

	// Also get sounds.json
	let sounds_json_entry = asset_index
		.objects
		.get("minecraft/sounds.json")
		.cloned();

	let total_files = sound_files.len() as u32 + if sounds_json_entry.is_some() { 1 } else { 0 };
	let total_bytes: u64 = sound_files.iter().map(|(_, obj)| obj.size).sum::<u64>()
		+ sounds_json_entry.as_ref().map(|e| e.size).unwrap_or(0);

	let client = reqwest::Client::new();
	let mut downloaded_files = 0u32;
	let mut downloaded_bytes = 0u64;
	let mut failed_files: Vec<String> = vec![];

	// Download sounds.json first
	if let Some(sj_entry) = &sounds_json_entry {
		let url = format!(
			"{}/{}/{}",
			RESOURCES_URL,
			&sj_entry.hash[..2],
			&sj_entry.hash
		);

		let mut success = false;
		for _ in 0..3 {
			match client.get(&url).send().await {
				Ok(resp) => {
					if let Ok(bytes) = resp.bytes().await {
						let sj_path = ver_dir.join("sounds.json");
						if std::fs::write(&sj_path, &bytes).is_ok() {
							success = true;
							downloaded_files += 1;
							downloaded_bytes += sj_entry.size;
							break;
						}
					}
				}
				Err(_) => continue,
			}
		}
		if !success {
			failed_files.push("minecraft/sounds.json".to_string());
		}

		let _ = app.emit(
			"download-progress",
			DownloadProgress {
				version_id: version_id.clone(),
				downloaded_files,
				total_files,
				downloaded_bytes,
				total_bytes,
				current_file: "sounds.json".to_string(),
				status: "downloading".to_string(),
			},
		);
	}

	// Download all sound files
	for (key, obj) in &sound_files {
		let url = format!("{}/{}/{}", RESOURCES_URL, &obj.hash[..2], &obj.hash);

		// Determine local file path
		let rel_path = key.strip_prefix("minecraft/sounds/").unwrap_or(key);
		let file_path = ver_dir.join("sounds").join(rel_path);

		// Create parent directories
		if let Some(parent) = file_path.parent() {
			let _ = std::fs::create_dir_all(parent);
		}

		// Skip if already downloaded
		if file_path.exists() && file_path.metadata().map(|m| m.len()).unwrap_or(0) == obj.size {
			downloaded_files += 1;
			downloaded_bytes += obj.size;
			let _ = app.emit(
				"download-progress",
				DownloadProgress {
					version_id: version_id.clone(),
					downloaded_files,
					total_files,
					downloaded_bytes,
					total_bytes,
					current_file: rel_path.to_string(),
					status: "downloading".to_string(),
				},
			);
			continue;
		}

		let mut success = false;
		for _ in 0..3 {
			match client.get(&url).send().await {
				Ok(resp) => {
					if let Ok(bytes) = resp.bytes().await {
						if std::fs::write(&file_path, &bytes).is_ok() {
							success = true;
							downloaded_files += 1;
							downloaded_bytes += obj.size;
							break;
						}
					}
				}
				Err(_) => continue,
			}
		}

		if !success {
			failed_files.push(key.clone());
		}

		let _ = app.emit(
			"download-progress",
			DownloadProgress {
				version_id: version_id.clone(),
				downloaded_files,
				total_files,
				downloaded_bytes,
				total_bytes,
				current_file: rel_path.to_string(),
				status: "downloading".to_string(),
			},
		);
	}

	// Save download status
	let status = DownloadStatus {
		total_files,
		downloaded_files,
		total_bytes,
		downloaded_bytes,
		complete: failed_files.is_empty(),
		failed_files: failed_files.clone(),
	};

	let status_json = serde_json::to_string_pretty(&status)
		.map_err(|e| format!("Failed to serialize download status: {}", e))?;
	std::fs::write(ver_dir.join("download_status.json"), status_json)
		.map_err(|e| format!("Failed to write download status: {}", e))?;

	let _ = app.emit(
		"download-progress",
		DownloadProgress {
			version_id: version_id.clone(),
			downloaded_files,
			total_files,
			downloaded_bytes,
			total_bytes,
			current_file: String::new(),
			status: if failed_files.is_empty() {
				"complete".to_string()
			} else {
				"failed".to_string()
			},
		},
	);

	if !failed_files.is_empty() {
		return Err(format!(
			"Failed to download {} files",
			failed_files.len()
		));
	}

	Ok(())
}

#[tauri::command]
pub async fn get_sound_tree(
	version_id: String,
	_pack_id: Option<String>,
) -> Result<Vec<SoundNode>, String> {
	let ver_dir = version_dir(&version_id);
	let ai_path = ver_dir.join("asset_index.json");
	let sj_path = ver_dir.join("sounds.json");

	if !ai_path.exists() {
		return Err(format!(
			"Asset index not found for version {}. Download sounds first.",
			version_id
		));
	}

	// Load asset index
	let ai_content = std::fs::read_to_string(&ai_path)
		.map_err(|e| format!("Failed to read asset index: {}", e))?;
	let asset_index: AssetIndex = serde_json::from_str(&ai_content)
		.map_err(|e| format!("Failed to parse asset index: {}", e))?;

	// Load sounds.json (optional — some old versions may not have it)
	let sounds_json: Option<SoundsJson> = if sj_path.exists() {
		let sj_content = std::fs::read_to_string(&sj_path)
			.map_err(|e| format!("Failed to read sounds.json: {}", e))?;
		Some(
			serde_json::from_str(&sj_content)
				.map_err(|e| format!("Failed to parse sounds.json: {}", e))?,
		)
	} else {
		None
	};

	// Build reverse mapping: sound file path → (event name, is_stream, all variant paths)
	let mut file_to_event: HashMap<String, (String, bool, Vec<String>)> = HashMap::new();

	if let Some(ref sj) = sounds_json {
		for (event_name, event) in &sj.0 {
			let variant_paths: Vec<String> = event
				.sounds
				.iter()
				.map(|s| format!("minecraft/sounds/{}.ogg", s.name()))
				.collect();

			for sound in &event.sounds {
				let file_path = format!("minecraft/sounds/{}.ogg", sound.name());
				file_to_event.insert(
					file_path,
					(event_name.clone(), sound.is_stream(), variant_paths.clone()),
				);
			}
		}
	}

	// Filter sound files from asset index
	let mut sound_files: Vec<(String, &AssetObject)> = asset_index
		.objects
		.iter()
		.filter(|(key, _)| key.starts_with("minecraft/sounds/") && key.ends_with(".ogg"))
		.map(|(key, obj)| (key.clone(), obj))
		.collect();

	sound_files.sort_by(|(a, _), (b, _)| a.cmp(b));

	// Build directory tree
	let mut root_children: Vec<SoundNode> = vec![];

	for (key, obj) in &sound_files {
		let rel_path = key.strip_prefix("minecraft/sounds/").unwrap_or(key);
		let parts: Vec<&str> = rel_path.split('/').collect();

		insert_into_tree(
			&mut root_children,
			&parts,
			key,
			obj,
			&file_to_event,
		);
	}

	// Sort tree recursively
	sort_tree(&mut root_children);

	Ok(root_children)
}

fn insert_into_tree(
	children: &mut Vec<SoundNode>,
	parts: &[&str],
	full_path: &str,
	obj: &AssetObject,
	file_to_event: &HashMap<String, (String, bool, Vec<String>)>,
) {
	if parts.is_empty() {
		return;
	}

	if parts.len() == 1 {
		// This is a file
		let (sound_event, is_long, variants) = file_to_event
			.get(full_path)
			.map(|(e, s, v)| (Some(e.clone()), *s, Some(v.clone())))
			.unwrap_or((None, false, None));

		children.push(SoundNode {
			name: parts[0].to_string(),
			path: full_path.to_string(),
			node_type: "file".to_string(),
			children: None,
			hash: Some(obj.hash.clone()),
			size: Some(obj.size),
			is_long_sound: is_long,
			sound_event,
			variants,
		});
	} else {
		// This is a directory segment
		let dir_name = parts[0];
		let dir_path = {
			// Compute directory path by trimming the remaining segments from full_path
			let remaining_len: usize =
				parts[1..].iter().map(|p| p.len()).sum::<usize>() + (parts.len() - 1);
			full_path[..full_path.len() - remaining_len]
				.trim_end_matches('/')
				.to_string()
		};

		// Find or create directory node
		let existing = children.iter_mut().find(|c| c.name == dir_name && c.node_type == "directory");

		if let Some(dir_node) = existing {
			let dir_children = dir_node.children.get_or_insert_with(Vec::new);
			insert_into_tree(
				dir_children,
				&parts[1..],
				full_path,
				obj,
				file_to_event,
			);
		} else {
			let mut new_children = vec![];
			insert_into_tree(
				&mut new_children,
				&parts[1..],
				full_path,
				obj,
				file_to_event,
			);

			children.push(SoundNode {
				name: dir_name.to_string(),
				path: dir_path,
				node_type: "directory".to_string(),
				children: Some(new_children),
				hash: None,
				size: None,
				is_long_sound: false,
				sound_event: None,
				variants: None,
			});
		}
	}
}

fn sort_tree(nodes: &mut Vec<SoundNode>) {
	nodes.sort_by(|a, b| {
		// Directories first, then files
		let a_is_dir = a.node_type == "directory";
		let b_is_dir = b.node_type == "directory";
		b_is_dir.cmp(&a_is_dir).then(a.name.cmp(&b.name))
	});

	for node in nodes.iter_mut() {
		if let Some(ref mut children) = node.children {
			sort_tree(children);
		}
	}
}

pub fn get_pack_format_for_version(version_id: &str) -> Result<u32, String> {
	let pv_path = version_dir(version_id).join("pack_version.json");
	if pv_path.exists() {
		let content = std::fs::read_to_string(&pv_path)
			.map_err(|e| format!("Failed to read pack version: {}", e))?;
		let pv: PackVersion = serde_json::from_str(&content)
			.map_err(|e| format!("Failed to parse pack version: {}", e))?;
		Ok(pv.resource.unwrap_or(1))
	} else {
		// Fallback: try to derive from version ID
		Ok(fallback_pack_format(version_id))
	}
}

fn fallback_pack_format(version_id: &str) -> u32 {
	// Fallback mapping for versions where pack_version.json might not exist
	let id = version_id;
	if id.starts_with("1.21") { 34 }
	else if id.starts_with("1.20.5") || id.starts_with("1.20.6") { 32 }
	else if id.starts_with("1.20.3") || id.starts_with("1.20.4") { 22 }
	else if id.starts_with("1.20.2") { 18 }
	else if id.starts_with("1.20") { 15 }
	else if id.starts_with("1.19.4") { 13 }
	else if id.starts_with("1.19.3") { 12 }
	else if id.starts_with("1.19") { 9 }
	else if id.starts_with("1.18") { 8 }
	else if id.starts_with("1.17") { 7 }
	else if id.starts_with("1.16.2") || id.starts_with("1.16.3") || id.starts_with("1.16.4") || id.starts_with("1.16.5") { 6 }
	else if id.starts_with("1.15") || id.starts_with("1.16") { 5 }
	else if id.starts_with("1.13") || id.starts_with("1.14") { 4 }
	else if id.starts_with("1.11") || id.starts_with("1.12") { 3 }
	else if id.starts_with("1.9") || id.starts_with("1.10") { 2 }
	else { 1 }
}

#[tauri::command]
pub async fn get_original_sound_path(
	version_id: String,
	sound_path: String,
) -> Result<String, String> {
	let rel_path = sound_path
		.strip_prefix("minecraft/sounds/")
		.unwrap_or(&sound_path);
	let file_path = version_dir(&version_id).join("sounds").join(rel_path);

	if file_path.exists() {
		Ok(file_path.to_string_lossy().to_string())
	} else {
		Err(format!("Original sound not found: {}", sound_path))
	}
}
