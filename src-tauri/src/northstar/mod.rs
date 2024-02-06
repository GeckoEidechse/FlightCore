//! This module deals with handling things around Northstar such as
//! - getting version number
pub mod install;
pub mod profile;

use crate::util::check_ea_app_or_origin_running;
use crate::{
    constants::{CORE_MODS, TITANFALL2_STEAM_ID},
    platform_specific::get_host_os,
    GameInstall, InstallType,
};
use crate::{NorthstarThunderstoreRelease, NorthstarThunderstoreReleaseWrapper};
use anyhow::anyhow;

/// Gets list of available Northstar versions from Thunderstore
#[tauri::command]
pub async fn get_available_northstar_versions(
) -> Result<Vec<NorthstarThunderstoreReleaseWrapper>, ()> {
    let northstar_package_name = "Northstar";
    let index = thermite::api::get_package_index().unwrap().to_vec();
    let nsmod = index
        .iter()
        .find(|f| f.name.to_lowercase() == northstar_package_name.to_lowercase())
        .ok_or_else(|| panic!("Couldn't find Northstar on thunderstore???"))
        .unwrap();

    let mut releases: Vec<NorthstarThunderstoreReleaseWrapper> = vec![];
    for (_version_string, nsmod_version_obj) in nsmod.versions.iter() {
        let current_elem = NorthstarThunderstoreRelease {
            package: nsmod_version_obj.name.clone(),
            version: nsmod_version_obj.version.clone(),
        };
        let current_elem_wrapped = NorthstarThunderstoreReleaseWrapper {
            label: format!(
                "{} v{}",
                nsmod_version_obj.name.clone(),
                nsmod_version_obj.version.clone()
            ),
            value: current_elem,
        };

        releases.push(current_elem_wrapped);
    }

    releases.sort_by(|a, b| {
        // Parse version number
        let a_ver = semver::Version::parse(&a.value.version).unwrap();
        let b_ver = semver::Version::parse(&b.value.version).unwrap();
        b_ver.partial_cmp(&a_ver).unwrap() // Sort newest first
    });

    Ok(releases)
}

/// Checks if installed Northstar version is up-to-date
/// false -> Northstar install is up-to-date
/// true  -> Northstar install is outdated
#[tauri::command]
pub async fn check_is_northstar_outdated(
    game_install: GameInstall,
    northstar_package_name: Option<String>,
) -> Result<bool, String> {
    let northstar_package_name = match northstar_package_name {
        Some(northstar_package_name) => {
            if northstar_package_name.len() <= 1 {
                "Northstar".to_string()
            } else {
                northstar_package_name
            }
        }
        None => "Northstar".to_string(),
    };

    let index = match thermite::api::get_package_index() {
        Ok(res) => res.to_vec(),
        Err(err) => return Err(format!("Couldn't check if Northstar up-to-date: {err}")),
    };
    let nmod = index
        .iter()
        .find(|f| f.name.to_lowercase() == northstar_package_name.to_lowercase())
        .expect("Couldn't find Northstar on thunderstore???");
    // .ok_or_else(|| anyhow!("Couldn't find Northstar on thunderstore???"))?;

    let version_number = match get_northstar_version_number(game_install) {
        Ok(version_number) => version_number,
        Err(err) => {
            log::warn!("{}", err);
            // If we fail to get new version just assume we are up-to-date
            return Err(err);
        }
    };

    // Release candidate version numbers are different between `mods.json` and Thunderstore
    let version_number = crate::util::convert_release_candidate_number(version_number);

    if version_number != nmod.latest {
        log::info!("Installed Northstar version outdated");
        Ok(true)
    } else {
        log::info!("Installed Northstar version up-to-date");
        Ok(false)
    }
}

/// Check version number of a mod
pub fn check_mod_version_number(path_to_mod_folder: &str) -> Result<String, anyhow::Error> {
    let data = std::fs::read_to_string(format!("{path_to_mod_folder}/mod.json"))?;
    let parsed_json: serde_json::Value = serde_json::from_str(&data)?;

    let mod_version_number = match parsed_json.get("Version").and_then(|value| value.as_str()) {
        Some(version_number) => version_number,
        None => return Err(anyhow!("No version number found")),
    };

    log::info!("{}", mod_version_number);

    Ok(mod_version_number.to_string())
}

/// Returns the current Northstar version number as a string
#[tauri::command]
pub fn get_northstar_version_number(game_install: GameInstall) -> Result<String, String> {
    log::info!("{}", game_install.game_path);

    // TODO:
    // Check if NorthstarLauncher.exe exists and check its version number
    let initial_version_number = match check_mod_version_number(&format!(
        "{}/{}/mods/{}",
        game_install.game_path, game_install.profile, CORE_MODS[0]
    )) {
        Ok(version_number) => version_number,
        Err(err) => return Err(err.to_string()),
    };

    for core_mod in CORE_MODS {
        let current_version_number = match check_mod_version_number(&format!(
            "{}/{}/mods/{}",
            game_install.game_path, game_install.profile, core_mod
        )) {
            Ok(version_number) => version_number,
            Err(err) => return Err(err.to_string()),
        };
        if current_version_number != initial_version_number {
            // We have a version number mismatch
            return Err("Found version number mismatch".to_string());
        }
    }
    log::info!("All mods same version");

    Ok(initial_version_number)
}

/// Launches Northstar
#[tauri::command]
pub fn launch_northstar(
    game_install: GameInstall,
    launch_args: Vec<&str>,
    launch_via_steam: Option<bool>,
    bypass_checks: Option<bool>,
) -> Result<String, String> {
    dbg!(game_install.clone());
    println!("Launch arguments: {:?}", launch_args);

    let launch_via_steam = launch_via_steam.unwrap_or(false);
    if launch_via_steam {
        return launch_northstar_steam(game_install, launch_args, bypass_checks);
    }

    let host_os = get_host_os();

    // Explicitly fail early certain (currently) unsupported install setups
    if host_os != "windows" {
        if !matches!(game_install.install_type, InstallType::STEAM) {
            return Err(format!(
                "Not yet implemented for \"{}\" with Titanfall2 installed via \"{:?}\"",
                get_host_os(),
                game_install.install_type
            ));
        }

        return launch_northstar_steam(game_install, launch_args, bypass_checks);
    }

    let bypass_checks = bypass_checks.unwrap_or(false);

    // Only check guards if bypassing checks is not enabled
    if !bypass_checks {
        // Some safety checks before, should have more in the future
        if get_northstar_version_number(game_install.clone()).is_err() {
            return Err(anyhow!("Not all checks were met").to_string());
        }

        // Require EA App or Origin to be running to launch Northstar
        let ea_app_is_running = check_ea_app_or_origin_running();
        if !ea_app_is_running {
            return Err(
                anyhow!("EA App not running, start EA App before launching Northstar").to_string(),
            );
        }
    }

    // Switch to Titanfall2 directory for launching
    // NorthstarLauncher.exe expects to be run from that folder
    if std::env::set_current_dir(game_install.game_path.clone()).is_err() {
        // We failed to get to Titanfall2 directory
        return Err(anyhow!("Couldn't access Titanfall2 directory").to_string());
    }

    // Only Windows with Steam or Origin are supported at the moment
    if host_os == "windows"
        && (matches!(game_install.install_type, InstallType::STEAM)
            || matches!(game_install.install_type, InstallType::ORIGIN)
            || matches!(game_install.install_type, InstallType::UNKNOWN))
    {
        let ns_exe_path = format!("{}/NorthstarLauncher.exe", game_install.game_path);
        let ns_profile_arg = format!("-profile={}", game_install.profile);
        let mut arguments = vec!["/C", "start", "", &ns_exe_path, &ns_profile_arg];
        arguments.extend(launch_args);

        let _output = std::process::Command::new("C:\\Windows\\System32\\cmd.exe")
            .args(arguments)
            .spawn()
            .expect("failed to execute process");
        return Ok("Launched game".to_string());
    }

    Err(format!(
        "Not yet implemented for {:?} on {}",
        game_install.install_type,
        get_host_os()
    ))
}

/// Prepare Northstar and Launch through Steam using the Browser Protocol
pub fn launch_northstar_steam(
    game_install: GameInstall,
    launch_args: Vec<&str>,
    _bypass_checks: Option<bool>,
) -> Result<String, String> {
    if !matches!(game_install.install_type, InstallType::STEAM) {
        return Err("Titanfall2 was not installed via Steam".to_string());
    }

    match steamlocate::SteamDir::locate() {
        Some(mut steamdir) => {
            if get_host_os() != "windows" {
                let titanfall2_steamid: u32 = TITANFALL2_STEAM_ID.parse().unwrap();
                match steamdir.compat_tool(&titanfall2_steamid) {
                    Some(_) => {}
                    None => {
                        return Err(
                            "Titanfall2 was not configured to use a compatibility tool".to_string()
                        );
                    }
                }
            }
        }
        None => {
            return Err("Couldn't access Titanfall2 directory".to_string());
        }
    }

    // Switch to Titanfall2 directory to set everything up
    if std::env::set_current_dir(game_install.game_path).is_err() {
        // We failed to get to Titanfall2 directory
        return Err("Couldn't access Titanfall2 directory".to_string());
    }

    match open::that(format!(
        "steam://run/{}//-profile={} --northstar/ {}",
        TITANFALL2_STEAM_ID,
        game_install.profile,
        launch_args.join(" ")
    )) {
        Ok(()) => Ok("Started game".to_string()),
        Err(_err) => Err("Failed to launch Titanfall 2 via Steam".to_string()),
    }
}
