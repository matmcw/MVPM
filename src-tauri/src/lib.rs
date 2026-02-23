mod commands;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_fs::init())
		.setup(|app| {
			if cfg!(debug_assertions) {
				app.handle().plugin(
					tauri_plugin_log::Builder::default()
						.level(log::LevelFilter::Info)
						.build(),
				)?;
			}
			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			// Mojang
			commands::mojang::fetch_version_manifest,
			commands::mojang::download_version_sounds,
			commands::mojang::is_version_downloaded,
			commands::mojang::get_download_status,
			commands::mojang::get_sound_tree,
			commands::mojang::get_original_sound_path,
			// Packs
			commands::packs::list_packs,
			commands::packs::create_pack,
			commands::packs::get_pack,
			commands::packs::update_pack,
			commands::packs::duplicate_pack,
			commands::packs::delete_pack,
			commands::packs::change_pack_version,
			commands::packs::get_recorded_sounds,
			commands::packs::get_pack_sound_path,
			// Recording
			commands::recording::save_recording,
			commands::recording::delete_recording,
			// Settings
			commands::settings::get_settings,
			commands::settings::save_settings,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
