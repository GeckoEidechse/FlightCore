use crate::constants::BLACKLISTED_MODS;
use crate::mod_management::{
    delete_mod_folder, get_installed_mods_and_properties, ParsedThunderstoreModString,
};
use crate::GameInstall;
use crate::NorthstarMod;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{io::Read, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModJson {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "ThunderstoreModString")]
    thunderstore_mod_string: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
}

/// Parses `manifest.json` for Thunderstore mod string
fn parse_for_thunderstore_mod_string(nsmod_path: &str) -> Result<String, anyhow::Error> {
    let manifest_json_path = format!("{}/manifest.json", nsmod_path);
    let ts_author_txt_path = format!("{}/thunderstore_author.txt", nsmod_path);

    // Check if `manifest.json` exists and parse
    let data = std::fs::read_to_string(manifest_json_path)?;
    let thunderstore_manifest: super::ThunderstoreManifest = json5::from_str(&data)?;

    // Check if `thunderstore_author.txt` exists and parse
    let mut file = std::fs::File::open(ts_author_txt_path)?;
    let mut thunderstore_author = String::new();
    file.read_to_string(&mut thunderstore_author)?;

    // Build mod string
    let thunderstore_mod_string = format!(
        "{}-{}-{}",
        thunderstore_author, thunderstore_manifest.name, thunderstore_manifest.version_number
    );

    Ok(thunderstore_mod_string)
}

/// Parse `mods` folder for installed mods.
pub fn parse_installed_mods(
    game_install: &GameInstall,
) -> Result<Vec<NorthstarMod>, anyhow::Error> {
    let ns_mods_folder = format!("{}/{}/mods/", game_install.game_path, game_install.profile);

    let paths = match std::fs::read_dir(ns_mods_folder) {
        Ok(paths) => paths,
        Err(_err) => return Err(anyhow!("No mods folder found")),
    };

    let mut directories: Vec<PathBuf> = Vec::new();
    let mut mods: Vec<NorthstarMod> = Vec::new();

    // Get list of folders in `mods` directory
    for path in paths {
        log::info!("{path:?}");
        let my_path = path.unwrap().path();
        log::info!("{my_path:?}");

        let md = std::fs::metadata(my_path.clone()).unwrap();
        if md.is_dir() {
            directories.push(my_path);
        }
    }

    // Iterate over folders and check if they are Northstar mods
    for directory in directories {
        let directory_str = directory.to_str().unwrap().to_string();
        // Check if mod.json exists
        let mod_json_path = format!("{}/mod.json", directory_str);
        if !std::path::Path::new(&mod_json_path).exists() {
            continue;
        }

        // Parse mod.json and get mod name

        // Read file into string and parse it
        let data = std::fs::read_to_string(mod_json_path.clone())?;
        let parsed_mod_json: ModJson = match json5::from_str(&data) {
            Ok(parsed_json) => parsed_json,
            Err(err) => {
                log::warn!("Failed parsing {} with {}", mod_json_path, err.to_string());
                continue;
            }
        };
        // Get Thunderstore mod string if it exists
        let mut thunderstore_mod_string = match parsed_mod_json.thunderstore_mod_string {
            // Attempt legacy method for getting Thunderstore string first
            Some(ts_mod_string) => Some(ts_mod_string),
            // Legacy method failed
            None => match parse_for_thunderstore_mod_string(&directory_str) {
                Ok(thunderstore_mod_string) => Some(thunderstore_mod_string),
                Err(_err) => None,
            },
        };
        // Get directory path
        let mod_directory = directory.to_str().unwrap().to_string();

        // This is a stupid way to show a legacy installed mod as outdated by simply giving back a wrong version number
        if thunderstore_mod_string.is_some() {
            // Parse the string
            let mut parsed_string: ParsedThunderstoreModString =
                thunderstore_mod_string.clone().unwrap().parse().unwrap();
            // Set version number to `0.0.0`
            parsed_string.version = "0.0.0".to_string();
            // And store new string back in original variable
            thunderstore_mod_string = Some(parsed_string.to_string())
        }

        let ns_mod = NorthstarMod {
            name: parsed_mod_json.name,
            version: parsed_mod_json.version,
            thunderstore_mod_string,
            enabled: false, // Placeholder
            directory: mod_directory,
        };

        mods.push(ns_mod);
    }

    // Return found mod names
    Ok(mods)
}

/// Deletes all legacy packages that match in author and mod name
/// regardless of version
///
/// "legacy package" refers to a Thunderstore package installed into the `mods` folder
/// by extracting Northstar mods contained inside and then adding `manifest.json` and `thunderstore_author.txt`
/// to indicate which Thunderstore package they are part of
pub fn delete_legacy_package_install(
    thunderstore_mod_string: &str,
    game_install: &GameInstall,
) -> Result<(), String> {
    let thunderstore_mod_string: ParsedThunderstoreModString =
        thunderstore_mod_string.parse().unwrap();
    let found_installed_legacy_mods = match parse_installed_mods(game_install) {
        Ok(res) => res,
        Err(err) => return Err(err.to_string()),
    };

    for legacy_mod in found_installed_legacy_mods {
        if legacy_mod.thunderstore_mod_string.is_none() {
            continue; // Not a thunderstore mod
        }

        let current_mod_ts_string: ParsedThunderstoreModString = legacy_mod
            .clone()
            .thunderstore_mod_string
            .unwrap()
            .parse()
            .unwrap();

        if thunderstore_mod_string.author_name == current_mod_ts_string.author_name
            && thunderstore_mod_string.mod_name == current_mod_ts_string.mod_name
        {
            // They match, delete
            delete_mod_folder(&legacy_mod.directory)?;
        }
    }

    Ok(())
}

/// Deletes all NorthstarMods related to a Thunderstore mod
pub fn delete_thunderstore_mod(
    game_install: GameInstall,
    thunderstore_mod_string: String,
) -> Result<(), String> {
    // Prevent deleting core mod
    for core_ts_mod in BLACKLISTED_MODS {
        if thunderstore_mod_string == core_ts_mod {
            return Err(format!("Cannot remove core mod {thunderstore_mod_string}"));
        }
    }

    let parsed_ts_mod_string: ParsedThunderstoreModString =
        thunderstore_mod_string.parse().unwrap();

    // Get installed mods
    let installed_ns_mods = get_installed_mods_and_properties(game_install)?;

    // List of mod folders to remove
    let mut mod_folders_to_remove: Vec<String> = Vec::new();

    // Get folder name based on Thundestore mod string
    for installed_ns_mod in installed_ns_mods {
        if installed_ns_mod.thunderstore_mod_string.is_none() {
            // Not a Thunderstore mod
            continue;
        }

        let installed_ns_mod_ts_string: ParsedThunderstoreModString = installed_ns_mod
            .thunderstore_mod_string
            .unwrap()
            .parse()
            .unwrap();

        // Installed mod matches specified Thunderstore mod string
        if parsed_ts_mod_string.author_name == installed_ns_mod_ts_string.author_name
            && parsed_ts_mod_string.mod_name == installed_ns_mod_ts_string.mod_name
        {
            // Add folder to list of folder to remove
            mod_folders_to_remove.push(installed_ns_mod.directory);
        }
    }

    if mod_folders_to_remove.is_empty() {
        return Err(format!(
            "No mods removed as no Northstar mods matching {thunderstore_mod_string} were found to be installed."
        ));
    }

    // Delete given folders
    for mod_folder in mod_folders_to_remove {
        delete_mod_folder(&mod_folder)?;
    }

    Ok(())
}
