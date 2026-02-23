use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Settings ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
	pub record_key: String,
	pub microphone_id: Option<String>,
	pub auto_play_original: bool,
	pub single_recording_mode: bool,
	pub packs_folder: Option<String>,
	pub theme: String,
	pub window_width: Option<u32>,
	pub window_height: Option<u32>,
	pub window_x: Option<i32>,
	pub window_y: Option<i32>,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			record_key: "Space".to_string(),
			microphone_id: None,
			auto_play_original: true,
			single_recording_mode: false,
			packs_folder: None,
			theme: "light".to_string(),
			window_width: None,
			window_height: None,
			window_x: None,
			window_y: None,
		}
	}
}

// --- Mojang API Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
	pub latest: LatestVersions,
	pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
	pub release: String,
	pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionEntry {
	pub id: String,
	#[serde(alias = "type")]
	pub version_type: String,
	pub url: String,
	pub time: String,
	pub release_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionJson {
	pub id: String,
	pub asset_index: AssetIndexRef,
	#[serde(default)]
	pub pack_version: Option<PackVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexRef {
	pub id: String,
	pub sha1: String,
	pub size: u64,
	#[serde(rename = "totalSize")]
	pub total_size: u64,
	pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackVersion {
	#[serde(default)]
	pub resource: Option<u32>,
	#[serde(default)]
	pub data: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
	pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetObject {
	pub hash: String,
	pub size: u64,
}

// --- sounds.json types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundsJson(pub HashMap<String, SoundEvent>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEvent {
	pub sounds: Vec<SoundEntry>,
	#[serde(default)]
	pub subtitle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SoundEntry {
	Simple(String),
	Detailed(SoundEntryDetailed),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEntryDetailed {
	pub name: String,
	#[serde(default)]
	pub stream: Option<bool>,
	#[serde(default)]
	pub volume: Option<f32>,
	#[serde(default)]
	pub pitch: Option<f32>,
	#[serde(default)]
	pub weight: Option<u32>,
}

impl SoundEntry {
	pub fn name(&self) -> &str {
		match self {
			SoundEntry::Simple(s) => s,
			SoundEntry::Detailed(d) => &d.name,
		}
	}

	pub fn is_stream(&self) -> bool {
		match self {
			SoundEntry::Simple(_) => false,
			SoundEntry::Detailed(d) => d.stream.unwrap_or(false),
		}
	}
}

// --- Sound Tree (sent to frontend) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundNode {
	pub name: String,
	pub path: String,
	pub node_type: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub children: Option<Vec<SoundNode>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hash: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub size: Option<u64>,
	pub is_long_sound: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub sound_event: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub variants: Option<Vec<String>>,
}

// --- Pack Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMeta {
	pub id: String,
	pub name: String,
	pub description: String,
	pub version_id: String,
	pub pack_format: u32,
	pub has_icon: bool,
	pub recorded_sounds: Vec<String>,
	pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackMcmeta {
	pub pack: PackMcmetaInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackMcmetaInner {
	pub pack_format: u32,
	pub description: String,
}

// --- Download Status ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStatus {
	pub total_files: u32,
	pub downloaded_files: u32,
	pub total_bytes: u64,
	pub downloaded_bytes: u64,
	pub complete: bool,
	pub failed_files: Vec<String>,
}

// --- Download Progress Event ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
	pub version_id: String,
	pub downloaded_files: u32,
	pub total_files: u32,
	pub downloaded_bytes: u64,
	pub total_bytes: u64,
	pub current_file: String,
	pub status: String,
}
