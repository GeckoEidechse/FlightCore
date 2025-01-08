// This file contains various mod management functions

use crate::constants::{
    BLACKLISTED_MODS, CORE_MODS, MODS_WITH_SPECIAL_REQUIREMENTS, NORTHSTAR_MODS_MANIFEST_VERSION,
};
use async_recursion::async_recursion;
use thermite::prelude::ThermiteError;

use crate::NorthstarMod;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;
use std::string::ToString;
use std::{fs, path::PathBuf};

mod legacy;
mod plugins;
use crate::GameInstall;

#[derive(Debug, Clone)]
pub struct ParsedThunderstoreModString {
    author_name: String,
    mod_name: String,
    version: String,
}

impl std::str::FromStr for ParsedThunderstoreModString {
    type Err = &'static str; // todo use an better error management

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check whether Thunderstore string passes regex
        let re = regex::Regex::new(r"^[a-zA-Z0-9_]+-[a-zA-Z0-9_]+-\d+\.\d+\.\d++$").unwrap();
        if !re.is_match(s) {
            return Err("Incorrect format");
        }

        let mut parts = s.split('-');

        let author_name = parts.next().ok_or("None value on author_name")?.to_string();
        let mod_name = parts.next().ok_or("None value on mod_name")?.to_string();
        let version = parts.next().ok_or("None value on version")?.to_string();

        Ok(ParsedThunderstoreModString {
            author_name,
            mod_name,
            version,
        })
    }
}

impl std::fmt::Display for ParsedThunderstoreModString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.author_name, self.mod_name, self.version)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThunderstoreManifest {
    name: String,
    version_number: String,
}

/// A wrapper around a temporary file handle and its path.
///
/// This struct is designed to be used for temporary files that should be automatically deleted
/// when the `TempFile` instance goes out of scope.
#[derive(Debug)]
pub struct TempFile(fs::File, PathBuf);

impl TempFile {
    pub fn new(file: fs::File, path: PathBuf) -> Self {
        Self(file, path)
    }

    pub fn file(&self) -> &fs::File {
        &self.0
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        _ = fs::remove_file(&self.1)
    }
}

impl std::ops::Deref for TempFile {
    type Target = fs::File;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Installs the specified mod
#[tauri::command]
pub async fn install_mod_wrapper(
    game_install: GameInstall,
    thunderstore_mod_string: String,
) -> Result<(), String> {
    match fc_download_mod_and_install(&game_install, &thunderstore_mod_string).await {
        Ok(()) => (),
        Err(err) => {
            log::warn!("{err}");
            return Err(err);
        }
    };
    match crate::repair_and_verify::clean_up_download_folder(&game_install, false) {
        Ok(()) => Ok(()),
        Err(err) => {
            log::info!("Failed to delete download folder due to {}", err);
            // Failure to delete download folder is not an error in mod install
            // As such ignore. User can still force delete if need be
            Ok(())
        }
    }
}

/// Returns a serde json object of the parsed `enabledmods.json` file
pub fn get_enabled_mods(game_install: &GameInstall) -> Result<serde_json::value::Value, String> {
    let enabledmods_json_path = format!(
        "{}/{}/enabledmods.json",
        game_install.game_path, game_install.profile
    );

    // Check for JSON file
    if !std::path::Path::new(&enabledmods_json_path).exists() {
        return Err("enabledmods.json not found".to_string());
    }

    // Read file
    let data = match std::fs::read_to_string(enabledmods_json_path) {
        Ok(data) => data,
        Err(err) => return Err(err.to_string()),
    };

    // Parse JSON
    let res: serde_json::Value = match serde_json::from_str(&data) {
        Ok(result) => result,
        Err(err) => return Err(format!("Failed to read JSON due to: {}", err)),
    };

    // Return parsed data
    Ok(res)
}

/// Gets all currently installed and enabled/disabled mods to rebuild `enabledmods.json`
pub fn rebuild_enabled_mods_json(
    game_install: &GameInstall,
    mut manifest_version: i64,
) -> Result<(), String> {
    let enabledmods_json_path = format!(
        "{}/{}/enabledmods.json",
        game_install.game_path, game_install.profile
    );
    let mods_and_properties = get_installed_mods_and_properties(game_install.clone())?;

    // Create new mapping
    let mut my_map = serde_json::Map::new();

    // Assign manifest version
    if manifest_version < 0 {
        manifest_version = NORTHSTAR_MODS_MANIFEST_VERSION;
    }
    my_map.insert("Version".to_string(), serde_json::json!(manifest_version));

    // Build mapping (adapting to manifest version)
    match manifest_version {
        0 => {
            for ns_mod in mods_and_properties.into_iter() {
                my_map.insert(ns_mod.name, serde_json::Value::Bool(ns_mod.enabled));
            }
        }
        1 => {
            for ns_mod in mods_and_properties.into_iter() {
                // Create mod key if needed
                if !my_map.contains_key(&ns_mod.name) {
                    let mut versions_map = serde_json::Map::new();
                    versions_map.insert(
                        ns_mod.version.unwrap(),
                        serde_json::Value::Bool(ns_mod.enabled),
                    );
                    my_map.insert(ns_mod.name, serde_json::Value::Object(versions_map));
                }
                // Else create version key
                else {
                    let mut version_map = my_map[&ns_mod.name].as_object().unwrap().clone();
                    version_map.insert(
                        ns_mod.version.unwrap(),
                        serde_json::Value::Bool(ns_mod.enabled),
                    );
                    my_map.insert(ns_mod.name, serde_json::Value::Object(version_map));
                }
            }
        }
        _ => return Err("Unknown manifest version.".to_string()),
    }

    let obj = serde_json::Value::Object(my_map);

    // Write to file
    std::fs::write(
        enabledmods_json_path,
        serde_json::to_string_pretty(&obj).unwrap(),
    )
    .unwrap();

    Ok(())
}

/// Set the status of a passed mod to enabled/disabled
#[tauri::command]
pub fn set_mod_enabled_status(
    game_install: GameInstall,
    mod_name: String,
    mut mod_version: String,
    is_enabled: bool,
) -> Result<(), String> {
    let enabledmods_json_path = format!(
        "{}/{}/enabledmods.json",
        game_install.game_path, game_install.profile
    );

    // Parse JSON
    let mut res: serde_json::Value = match get_enabled_mods(&game_install) {
        Ok(res) => res,
        Err(err) => {
            log::warn!("Couldn't parse `enabledmod.json`: {}", err);
            log::warn!("Rebuilding file.");

            rebuild_enabled_mods_json(&game_install, -1)?;

            // Then try again
            get_enabled_mods(&game_install)?
        }
    };

    // Check if key exists
    if res.get(mod_name.clone()).is_none() {
        // If it doesn't exist, rebuild `enabledmod.json`
        log::info!("Value not found in `enabledmod.json`. Rebuilding file");

        let manifest_version =
            match res.get("Version").is_some() && res.get("Version").unwrap().is_i64() {
                true => res.get("Version").unwrap().as_i64().unwrap(),
                false => NORTHSTAR_MODS_MANIFEST_VERSION,
            };

        rebuild_enabled_mods_json(&game_install, manifest_version)?;

        // Then try again
        res = get_enabled_mods(&game_install)?;
    }

    // Get manifest format version
    let mut manifest_version = NORTHSTAR_MODS_MANIFEST_VERSION;
    let manifest_object = &res.as_object().unwrap();

    if manifest_object.contains_key("Version") && manifest_object["Version"].is_number() {
        manifest_version = manifest_object["Version"].as_i64().unwrap();
    }
    log::info!(
        "Using enabledmods.json format version {}.",
        manifest_version
    );

    // Fail without version parameter
    if mod_version.is_empty() {
        match manifest_version {
            0 => (),
            _ => {
                // Trying to retrieve version
                let mods_and_properties = get_installed_mods_and_properties(game_install.clone())?;
                for m in mods_and_properties {
                    if m.name == mod_name {
                        mod_version = m.version.unwrap();
                        break;
                    }
                }

                // Fail if version couldn't be retrieved
                if mod_version.is_empty() {
                    log::error!("Didn't find mod version for mod \"{}\".", mod_name);
                    return Err(
                        "todo: Missing `mod_version` parameter with new enabledmods.json format."
                            .to_string(),
                    );
                }
            }
        }
    }

    // Update value
    match manifest_version {
        0 => {
            res[mod_name] = serde_json::Value::Bool(is_enabled);
        }
        _ => {
            // Create mod entry if needed
            if !res.as_object().unwrap().contains_key(&mod_name) || !res[&mod_name].is_object() {
                let mut version_map: serde_json::Map<String, serde_json::Value> =
                    serde_json::Map::new();
                version_map.insert(mod_version, serde_json::Value::Bool(is_enabled));
                res[&mod_name] = serde_json::Value::Object(version_map);
            } else {
                res[mod_name][mod_version] = serde_json::Value::Bool(is_enabled);
            }
        }
    }

    // Save the JSON structure into the output file
    std::fs::write(
        enabledmods_json_path,
        serde_json::to_string_pretty(&res).unwrap(),
    )
    .unwrap();

    Ok(())
}

/// Resembles the bare minimum keys in Northstar `mods.json`
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModJson {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Version")]
    version: Option<String>,
}

/// Parse `mods` folder for installed mods.
pub fn parse_mods_in_package(
    package_mods_path: PathBuf,
    thunderstore_mod_string: ParsedThunderstoreModString,
) -> Result<Vec<NorthstarMod>, anyhow::Error> {
    let paths = match std::fs::read_dir(package_mods_path) {
        Ok(paths) => paths,
        Err(_err) => return Err(anyhow!("No mods folder found")),
    };

    let mut directories: Vec<PathBuf> = Vec::new();
    let mut mods: Vec<NorthstarMod> = Vec::new();

    // Get list of folders in `mods` directory
    for path in paths {
        let my_path = path?.path();
        let md = std::fs::metadata(my_path.clone())?;
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

        // Read file into string and parse it
        let data = std::fs::read_to_string(mod_json_path.clone())?;
        let parsed_mod_json: ModJson = match json5::from_str(&data) {
            Ok(parsed_json) => parsed_json,
            Err(err) => {
                log::warn!("Failed parsing {} with {}", mod_json_path, err.to_string());
                continue;
            }
        };

        // Get directory path
        let mod_directory = directory.to_str().unwrap().to_string();

        let ns_mod = NorthstarMod {
            name: parsed_mod_json.name,
            version: parsed_mod_json.version,
            thunderstore_mod_string: Some(thunderstore_mod_string.to_string()),
            enabled: false, // Placeholder
            directory: mod_directory,
        };

        mods.push(ns_mod);
    }

    // Return found mod names
    Ok(mods)
}

/// Parse `packages` folder for installed mods.
pub fn parse_installed_package_mods(
    game_install: &GameInstall,
) -> Result<Vec<NorthstarMod>, anyhow::Error> {
    let mut collected_mods: Vec<NorthstarMod> = Vec::new();

    let packages_folder = format!(
        "{}/{}/packages/",
        game_install.game_path, game_install.profile
    );

    let packages_dir = match fs::read_dir(packages_folder) {
        Ok(res) => res,
        Err(err) => {
            // We couldn't read directory, probably cause it doesn't exist yet.
            // In that case we just say no package mods installed.
            log::warn!("{err}");
            return Ok(vec![]);
        }
    };

    // Iteratore over folders in `packages` dir
    for entry in packages_dir {
        let entry_path = entry?.path();
        let entry_str = entry_path.file_name().unwrap().to_str().unwrap();

        // Use the struct's from_str function to verify format
        if entry_path.is_dir() {
            let package_thunderstore_string = match ParsedThunderstoreModString::from_str(entry_str)
            {
                Ok(res) => res,
                Err(err) => {
                    log::warn!(
                        "Not a Thunderstore mod string \"{}\" cause: {}",
                        entry_path.display(),
                        err
                    );
                    continue;
                }
            };
            let manifest_path = entry_path.join("manifest.json");
            let mods_path = entry_path.join("mods");

            // Ensure `manifest.json` and `mods/` dir exist
            if manifest_path.exists() && mods_path.is_dir() {
                let mods =
                    match parse_mods_in_package(mods_path, package_thunderstore_string.clone()) {
                        Ok(res) => res,
                        Err(err) => {
                            log::warn!("Failed parsing cause: {err}");
                            continue;
                        }
                    };
                collected_mods.extend(mods);
            }
        }
    }

    Ok(collected_mods)
}

/// Gets list of installed mods and their properties
/// - name
/// - is enabled?
#[tauri::command]
pub fn get_installed_mods_and_properties(
    game_install: GameInstall,
) -> Result<Vec<NorthstarMod>, String> {
    // Get installed mods from packages
    let mut found_installed_mods = match parse_installed_package_mods(&game_install) {
        Ok(res) => res,
        Err(err) => return Err(err.to_string()),
    };
    // Get installed legacy mods
    let found_installed_legacy_mods = match legacy::parse_installed_mods(&game_install) {
        Ok(res) => res,
        Err(err) => return Err(err.to_string()),
    };

    // Combine list of package and legacy mods
    found_installed_mods.extend(found_installed_legacy_mods);

    // Get enabled mods as JSON
    let enabled_mods: serde_json::Value = match get_enabled_mods(&game_install) {
        Ok(enabled_mods) => enabled_mods,
        Err(_) => serde_json::from_str("{}").unwrap(), // `enabledmods.json` not found, create empty object
    };

    let mut installed_mods = Vec::new();
    let binding = serde_json::Map::new(); // Empty map in case treating as object fails
    let mapping = enabled_mods.as_object().unwrap_or(&binding);

    // With the change introduced in https://github.com/R2Northstar/NorthstarLauncher/pull/828,
    // enabledmods.json now has a different format, forcing us to detect whether it is used and
    // react accordingly.
    //
    let old_format_used: bool = !mapping.is_empty() && !mapping.contains_key("Version")
        || !mapping.is_empty()
            && mapping.contains_key("Version")
            && mapping.get("Version").unwrap() == 0;
    log::info!("Old enabledmods.json format detected: {old_format_used}");

    // Use list of installed mods and set enabled based on `enabledmods.json`
    for mut current_mod in found_installed_mods {
        let current_mod_enabled: bool = match old_format_used {
            // Old format: { modName: string => is_enabled: bool }
            true => match mapping.get(&current_mod.name) {
                Some(enabled) => enabled.as_bool().unwrap(),
                None => true, // Northstar considers mods not in mapping as enabled.
            },

            // New format: { modName: string => versions: string[] => is_enabled: bool }
            false => match mapping.get(&current_mod.name) {
                Some(versions) => match versions.get(current_mod.version.clone().unwrap()) {
                    Some(enabled) => enabled.as_bool().unwrap(),
                    None => false,
                },
                None => false,
            },
        };

        current_mod.enabled = current_mod_enabled;
        installed_mods.push(current_mod);
    }

    Ok(installed_mods)
}

async fn get_ns_mod_download_url(thunderstore_mod_string: &str) -> Result<String, String> {
    // TODO: This will crash the thread if not internet connection exist. `match` should be used instead
    let index = thermite::api::get_package_index().unwrap().to_vec();

    // Parse mod string
    let parsed_ts_mod_string: ParsedThunderstoreModString = match thunderstore_mod_string.parse() {
        Ok(res) => res,
        Err(_) => return Err("Failed to parse mod string".to_string()),
    };

    // Encode as URL
    let ts_mod_string_url = format!(
        "{}/{}/{}",
        parsed_ts_mod_string.author_name,
        parsed_ts_mod_string.mod_name,
        parsed_ts_mod_string.version
    );

    for ns_mod in index {
        // Iterate over all versions of a given mod
        for ns_mod in ns_mod.versions.values() {
            if ns_mod.url.contains(&ts_mod_string_url) {
                return Ok(ns_mod.url.clone());
            }
        }
    }

    Err("Could not find mod on Thunderstore".to_string())
}

/// Returns a vector of modstrings containing the dependencies of a given mod
async fn get_mod_dependencies(thunderstore_mod_string: &str) -> Result<Vec<String>, anyhow::Error> {
    log::info!("Attempting to get dependencies for: {thunderstore_mod_string}");

    let index = thermite::api::get_package_index()?.to_vec();

    // String replace works but more care should be taken in the future
    let ts_mod_string_url = thunderstore_mod_string.replace('-', "/");

    // Iterate over index
    for ns_mod in index {
        // Iterate over all versions of a given mod
        for ns_mod in ns_mod.versions.values() {
            if ns_mod.url.contains(&ts_mod_string_url) {
                return Ok(ns_mod.deps.clone());
            }
        }
    }
    Ok(Vec::<String>::new())
}

/// Deletes all versions of Thunderstore package except the specified one
fn delete_older_versions(
    thunderstore_mod_string: &str,
    game_install: &GameInstall,
) -> Result<(), String> {
    let thunderstore_mod_string: ParsedThunderstoreModString =
        thunderstore_mod_string.parse().unwrap();
    log::info!(
        "Deleting other versions of {}",
        thunderstore_mod_string.to_string()
    );
    let packages_folder = format!(
        "{}/{}/packages",
        game_install.game_path, game_install.profile
    );

    // Get folders in packages dir
    let paths = match std::fs::read_dir(&packages_folder) {
        Ok(paths) => paths,
        Err(_err) => return Err(format!("Failed to read directory {}", &packages_folder)),
    };

    let mut directories: Vec<PathBuf> = Vec::new();

    // Get list of folders in `mods` directory
    for path in paths {
        let my_path = path.unwrap().path();

        let md = std::fs::metadata(my_path.clone()).unwrap();
        if md.is_dir() {
            directories.push(my_path);
        }
    }

    for directory in directories {
        let folder_name = directory.file_name().unwrap().to_str().unwrap();
        let ts_mod_string_from_folder: ParsedThunderstoreModString = match folder_name.parse() {
            Ok(res) => res,
            Err(err) => {
                // Failed parsing folder name as Thunderstore mod string
                // This means it doesn't follow the `AUTHOR-MOD-VERSION` naming structure
                // This folder could've been manually created by the user or another application
                // As parsing failed we cannot determine the Thunderstore package it is part of hence we skip it
                log::warn!("{err}");
                continue;
            }
        };
        // Check which match `AUTHOR-MOD` and do NOT match `AUTHOR-MOD-VERSION`
        if ts_mod_string_from_folder.author_name == thunderstore_mod_string.author_name
            && ts_mod_string_from_folder.mod_name == thunderstore_mod_string.mod_name
            && ts_mod_string_from_folder.version != thunderstore_mod_string.version
        {
            delete_package_folder(&directory.display().to_string())?;
        }
    }

    Ok(())
}

/// Checks whether some mod is correctly formatted
/// Currently checks whether
/// - Some `mod.json` exists under `mods/*/mod.json`
fn fc_sanity_check(input: &&fs::File) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let mut archive = match zip::read::ZipArchive::new(*input) {
        Ok(archive) => archive,
        Err(_) => {
            return Err(Box::new(ThermiteError::UnknownError(
                "Failed reading zip file".into(),
            )))
        }
    };

    let mut has_mods = false;
    let mut mod_json_exists = false;

    // Checks for `mods/*/mod.json`
    for i in 0..archive.len() {
        let file = match archive.by_index(i) {
            Ok(file) => file,
            Err(_) => continue,
        };
        let file_path = file.mangled_name();
        if file_path.starts_with("mods/") {
            has_mods = true;
            if let Some(name) = file_path.file_name() {
                if name == "mod.json" {
                    let parent_path = file_path.parent().unwrap();
                    if parent_path.parent().unwrap().to_str().unwrap() == "mods" {
                        mod_json_exists = true;
                    }
                }
            }
        }

        if file_path.starts_with("plugins/") {
            if let Some(name) = file_path.file_name() {
                if name.to_str().unwrap().contains(".dll") {
                    log::warn!("Plugin detected, prompting user");
                    if !plugins::plugin_prompt() {
                        return Err(Box::new(ThermiteError::UnknownError(
                            "Plugin detected and install denied".into(),
                        )));
                    }
                }
            }
        }
    }

    if has_mods && mod_json_exists {
        Ok(())
    } else {
        Err(Box::new(ThermiteError::UnknownError(
            "Mod not correctly formatted".into(),
        )))
    }
}

// Copied from `libtermite` source code and modified
// Should be replaced with a library call to libthermite in the future
/// Download and install mod to the specified target.
#[async_recursion]
pub async fn fc_download_mod_and_install(
    game_install: &GameInstall,
    thunderstore_mod_string: &str,
) -> Result<(), String> {
    log::info!("Attempting to install \"{thunderstore_mod_string}\" to {game_install:?}");
    // Get mods and download directories
    let download_directory = format!(
        "{}/___flightcore-temp/download-dir/",
        game_install.game_path
    );

    // Early return on empty string
    if thunderstore_mod_string.is_empty() {
        return Err("Passed empty string".to_string());
    }

    let deps = match get_mod_dependencies(thunderstore_mod_string).await {
        Ok(deps) => deps,
        Err(err) => return Err(err.to_string()),
    };
    log::info!("Mod dependencies: {deps:?}");

    // Recursively install dependencies
    for dep in deps {
        match fc_download_mod_and_install(game_install, &dep).await {
            Ok(()) => (),
            Err(err) => {
                if err == "Cannot install Northstar as a mod!" {
                    continue; // For Northstar as a dependency, we just skip it
                } else {
                    return Err(err);
                }
            }
        };
    }

    // Prevent installing Northstar as a mod
    // While it would fail during install anyway, having explicit error message is nicer
    for blacklisted_mod in BLACKLISTED_MODS {
        if thunderstore_mod_string.contains(blacklisted_mod) {
            return Err("Cannot install Northstar as a mod!".to_string());
        }
    }

    // Prevent installing mods that have specific install requirements
    for special_mod in MODS_WITH_SPECIAL_REQUIREMENTS {
        if thunderstore_mod_string.contains(special_mod) {
            return Err(format!(
                "{} has special install requirements and cannot be installed with FlightCore",
                thunderstore_mod_string
            ));
        }
    }

    // Get download URL for the specified mod
    let download_url = get_ns_mod_download_url(thunderstore_mod_string).await?;

    // Create download directory
    match std::fs::create_dir_all(download_directory.clone()) {
        Ok(()) => (),
        Err(err) => return Err(err.to_string()),
    };

    let path = format!(
        "{}/___flightcore-temp/download-dir/{thunderstore_mod_string}.zip",
        game_install.game_path
    );

    // Download the mod
    let temp_file = TempFile::new(
        std::fs::File::options()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .map_err(|e| e.to_string())?,
        (&path).into(),
    );
    match thermite::core::manage::download(temp_file.file(), download_url) {
        Ok(_written_bytes) => (),
        Err(err) => return Err(err.to_string()),
    };

    // Get directory to install to made up of packages directory and Thunderstore mod string
    let install_directory = format!(
        "{}/{}/packages/",
        game_install.game_path, game_install.profile
    );

    // Extract the mod to the mods directory
    match thermite::core::manage::install_with_sanity(
        thunderstore_mod_string,
        temp_file.file(),
        std::path::Path::new(&install_directory),
        fc_sanity_check,
    ) {
        Ok(_) => (),
        Err(err) => {
            log::warn!("libthermite couldn't install mod {thunderstore_mod_string} due to {err:?}",);
            return match err {
                ThermiteError::SanityError(e) => Err(
                    format!("Mod failed sanity check during install. It's probably not correctly formatted. {}", e)
                ),
                _ => Err(err.to_string()),
            };
        }
    };

    // Successful package install
    match legacy::delete_legacy_package_install(thunderstore_mod_string, game_install) {
        Ok(()) => (),
        Err(err) => {
            // Catch error but ignore
            log::warn!("Failed deleting legacy versions due to: {}", err);
        }
    };

    match delete_older_versions(thunderstore_mod_string, game_install) {
        Ok(()) => (),
        Err(err) => {
            // Catch error but ignore
            log::warn!("Failed deleting older versions due to: {}", err);
        }
    };

    Ok(())
}

/// Deletes a given Northstar mod folder
fn delete_mod_folder(ns_mod_directory: &str) -> Result<(), String> {
    let ns_mod_dir_path = std::path::Path::new(&ns_mod_directory);

    // Safety check: Check whether `mod.json` exists and exit early if not
    // If it does not exist, we might not be dealing with a Northstar mod
    let mod_json_path = ns_mod_dir_path.join("mod.json");
    if !mod_json_path.exists() {
        // If it doesn't exist, return an error
        return Err(format!("mod.json does not exist in {}", ns_mod_directory));
    }

    match std::fs::remove_dir_all(ns_mod_directory) {
        Ok(()) => Ok(()),
        Err(err) => Err(format!("Failed deleting mod: {err}")),
    }
}

/// Deletes a Northstar mod based on its name
#[tauri::command]
pub fn delete_northstar_mod(game_install: GameInstall, nsmod_name: String) -> Result<(), String> {
    // Prevent deleting core mod
    for core_mod in CORE_MODS {
        if nsmod_name == core_mod {
            return Err(format!("Cannot remove core mod {nsmod_name}"));
        }
    }

    // Get installed mods
    let installed_ns_mods = get_installed_mods_and_properties(game_install)?;

    // Get folder name based on northstarmods
    for installed_ns_mod in installed_ns_mods {
        // Installed mod matches specified mod
        if installed_ns_mod.name == nsmod_name {
            // Delete folder
            return delete_mod_folder(&installed_ns_mod.directory);
        }
    }

    Err(format!("Mod {nsmod_name} not found to be installed"))
}

/// Deletes a given Thunderstore package
fn delete_package_folder(ts_package_directory: &str) -> Result<(), String> {
    let ns_mod_dir_path = std::path::Path::new(&ts_package_directory);

    // Safety check: Check whether `manifest.json` exists and exit early if not
    // If it does not exist, we might not be dealing with a Thunderstore package
    let mod_json_path = ns_mod_dir_path.join("manifest.json");
    if !mod_json_path.exists() {
        // If it doesn't exist, return an error
        return Err(format!(
            "manifest.json does not exist in {}",
            ts_package_directory
        ));
    }

    match std::fs::remove_dir_all(ts_package_directory) {
        Ok(()) => Ok(()),
        Err(err) => Err(format!("Failed deleting package: {err}")),
    }
}

/// Deletes all NorthstarMods related to a Thunderstore mod
#[tauri::command]
pub fn delete_thunderstore_mod(
    game_install: GameInstall,
    thunderstore_mod_string: String,
) -> Result<(), String> {
    // Check packages
    let packages_folder = format!(
        "{}/{}/packages",
        game_install.game_path, game_install.profile
    );
    if std::path::Path::new(&packages_folder).exists() {
        for entry in fs::read_dir(packages_folder).unwrap() {
            let entry = entry.unwrap();

            // Check if it's a folder and skip if otherwise
            if !entry.file_type().unwrap().is_dir() {
                log::warn!("Skipping \"{}\", not a file", entry.path().display());
                continue;
            }

            let entry_path = entry.path();
            let package_folder_ts_string = entry_path.file_name().unwrap().to_string_lossy();

            if package_folder_ts_string != thunderstore_mod_string {
                // Not the mod folder we are looking for, try the next one\
                continue;
            }

            // All checks passed, this is the matching mod
            return delete_package_folder(&entry.path().display().to_string());
        }
    }

    // Try legacy mod installs as fallback
    legacy::delete_thunderstore_mod(game_install, thunderstore_mod_string)
}
