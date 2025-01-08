use crate::constants::NORTHSTAR_MODS_MANIFEST_VERSION;
use crate::mod_management::{get_enabled_mods, rebuild_enabled_mods_json, set_mod_enabled_status};
/// Contains various functions to repair common issues and verifying installation
use crate::{constants::CORE_MODS, GameInstall};

/// Checks if is valid Titanfall2 install based on certain conditions
#[tauri::command]
pub async fn verify_install_location(game_path: String) -> bool {
    match check_is_valid_game_path(&game_path) {
        Ok(()) => true,
        Err(err) => {
            log::warn!("{}", err);
            false
        }
    }
}

/// Checks whether the provided path is a valid Titanfall2 gamepath by checking against a certain set of criteria
pub fn check_is_valid_game_path(game_install_path: &str) -> Result<(), String> {
    let path_to_titanfall2_exe = format!("{game_install_path}/Titanfall2.exe");
    let is_correct_game_path = std::path::Path::new(&path_to_titanfall2_exe).exists();
    log::info!("Titanfall2.exe exists in path? {}", is_correct_game_path);

    // Exit early if wrong game path
    if !is_correct_game_path {
        return Err(format!("Incorrect game path \"{game_install_path}\"")); // Return error cause wrong game path
    }
    Ok(())
}

/// Verifies Titanfall2 game files
#[tauri::command]
pub fn verify_game_files(game_install: GameInstall) -> Result<String, String> {
    dbg!(game_install);
    Err("TODO, not yet implemented".to_string())
}

/// Disables all mods except core ones
/// Enables core mods if disabled
#[tauri::command]
pub fn disable_all_but_core(game_install: GameInstall) -> Result<(), String> {
    // Try to fetch `enabledmods.json` a first time to try getting a manifest version
    let manifest_version: i64 = match get_enabled_mods(&game_install) {
        Ok(res) => match &res.as_object().unwrap().contains_key("Version") {
            false => NORTHSTAR_MODS_MANIFEST_VERSION,
            true => res
                .as_object()
                .unwrap()
                .get("Version")
                .unwrap()
                .as_i64()
                .unwrap(),
        },
        Err(_) => NORTHSTAR_MODS_MANIFEST_VERSION,
    };

    // Rebuild `enabledmods.json` to ensure all mods are added
    rebuild_enabled_mods_json(&game_install, manifest_version)?;

    let current_mods = get_enabled_mods(&game_install)?;

    // Disable all mods, set core mods to enabled
    for (key, _value) in current_mods.as_object().unwrap() {
        if CORE_MODS.contains(&key.as_str()) {
            // This is a core mod, we do not want to disable it
            set_mod_enabled_status(game_install.clone(), key.to_string(), "".to_string(), true)?;
        }
        // Not a core mod
        else if key != "Version" {
            // Disable all mod versions...
            if _value.is_object() {
                for version in _value.as_object().unwrap().keys() {
                    set_mod_enabled_status(
                        game_install.clone(),
                        key.to_string(),
                        version.clone(),
                        false,
                    )?;
                }
            }
            // ...or simply the mod version listed
            else {
                set_mod_enabled_status(
                    game_install.clone(),
                    key.to_string(),
                    _value.as_str().unwrap_or("").to_string(),
                    false,
                )?;
            }
        }
    }

    Ok(())
}

/// Installs the specified mod
#[tauri::command]
pub async fn clean_up_download_folder_wrapper(
    game_install: GameInstall,
    force: bool,
) -> Result<(), String> {
    match clean_up_download_folder(&game_install, force) {
        Ok(()) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

/// Deletes download folder
/// If `force` is FALSE, bails on non-empty folder
/// If `force` is TRUE, deletes folder even if non-empty
pub fn clean_up_download_folder(
    game_install: &GameInstall,
    force: bool,
) -> Result<(), anyhow::Error> {
    const TEMPORARY_DIRECTORIES: [&str; 4] = [
        "___flightcore-temp-download-dir",
        "___flightcore-temp/download-dir",
        "___flightcore-temp/extract-dir",
        "___flightcore-temp",
    ];

    for directory in TEMPORARY_DIRECTORIES {
        // Get download directory
        let download_directory = format!("{}/{}/", game_install.game_path, directory);

        // Check if files in folder
        let download_dir_contents = match std::fs::read_dir(download_directory.clone()) {
            Ok(contents) => contents,
            Err(_) => continue,
        };

        let mut count = 0;
        download_dir_contents.for_each(|_| count += 1);

        if count > 0 && !force {
            // Skip folder if not empty
            log::warn!("Folder not empty, not deleting: {directory}");
            continue;
        }

        // Delete folder
        std::fs::remove_dir_all(download_directory)?;
    }
    Ok(())
}

/// Get list of Northstar logs
#[tauri::command]
pub fn get_log_list(game_install: GameInstall) -> Result<Vec<std::path::PathBuf>, String> {
    let ns_log_folder = format!("{}/{}/logs", game_install.game_path, game_install.profile);

    // List files in logs folder
    let paths = match std::fs::read_dir(ns_log_folder) {
        Ok(paths) => paths,
        Err(_err) => return Err("No logs folder found".to_string()),
    };

    // Stores paths of log files
    let mut log_files: Vec<std::path::PathBuf> = Vec::new();

    for path in paths {
        let path = path.unwrap().path();
        if path.display().to_string().contains("nslog") {
            log_files.push(path);
        }
    }

    if !log_files.is_empty() {
        Ok(log_files)
    } else {
        Err("No logs found".to_string())
    }
}
