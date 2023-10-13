#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    env,
    sync::{Arc, Mutex},
    time::Duration,
};

mod constants;
mod development;
mod github;
mod mod_management;
mod northstar;
mod platform_specific;
mod repair_and_verify;
mod thunderstore;
mod util;

use semver::Version;
use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use tauri::api::dialog::blocking::MessageDialogBuilder;
#[cfg(target_os = "windows")]
use tauri::api::dialog::{MessageDialogButtons, MessageDialogKind};
use tauri::Manager;
use tokio::time::sleep;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
struct NorthstarThunderstoreRelease {
    package: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
struct NorthstarThunderstoreReleaseWrapper {
    label: String,
    value: NorthstarThunderstoreRelease,
}

#[derive(Default)]
struct Counter(Arc<Mutex<i32>>);

fn main() {
    // Setup logger
    let mut log_builder = pretty_env_logger::formatted_builder();
    log_builder.parse_filters("info");
    let logger = sentry_log::SentryLogger::with_dest(log_builder.build());

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    // Only enable Sentry crash logs on release
    #[cfg(not(debug_assertions))]
    let _guard = sentry::init((
        "https://f833732deb2240b0b2dc4abce97d0f1d@o1374052.ingest.sentry.io/6692177",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            attach_stacktrace: true,
            ..Default::default()
        },
    ));

    match tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_handle = app.app_handle();
            tauri::async_runtime::spawn(async move {
                loop {
                    sleep(Duration::from_millis(2000)).await;
                    // println!("sending backend ping");
                    app_handle.emit_all("backend-ping", "ping").unwrap();
                }
            });
            let app_handle = app.app_handle();
            tauri::async_runtime::spawn(async move {
                loop {
                    sleep(Duration::from_millis(2000)).await;
                    app_handle
                        .emit_all(
                            "ea-app-running-ping",
                            util::check_ea_app_or_origin_running(),
                        )
                        .unwrap();
                }
            });
            let app_handle = app.app_handle();
            tauri::async_runtime::spawn(async move {
                loop {
                    sleep(Duration::from_millis(2000)).await;
                    app_handle
                        .emit_all("northstar-running-ping", util::check_northstar_running())
                        .unwrap();
                }
            });

            // Emit updated player and server count to GUI
            let app_handle = app.app_handle();
            tauri::async_runtime::spawn(async move {
                loop {
                    sleep(constants::REFRESH_DELAY).await;
                    app_handle
                        .emit_all(
                            "northstar-statistics",
                            util::get_server_player_count().await,
                        )
                        .unwrap();
                }
            });

            Ok(())
        })
        .manage(Counter(Default::default()))
        .invoke_handler(tauri::generate_handler![
            util::force_panic,
            northstar::install::find_game_install_location,
            util::get_flightcore_version_number,
            northstar::get_northstar_version_number,
            check_is_northstar_outdated,
            repair_and_verify::verify_install_location,
            platform_specific::get_host_os,
            northstar::install::install_northstar_wrapper,
            northstar::install::update_northstar,
            northstar::launch_northstar,
            northstar::launch_northstar_steam,
            github::release_notes::check_is_flightcore_outdated,
            repair_and_verify::get_log_list,
            repair_and_verify::verify_game_files,
            mod_management::set_mod_enabled_status,
            repair_and_verify::disable_all_but_core,
            util::is_debug_mode,
            github::release_notes::get_northstar_release_notes,
            platform_specific::linux_checks,
            mod_management::get_installed_mods_and_properties,
            install_mod_wrapper,
            clean_up_download_folder_wrapper,
            github::release_notes::get_newest_flightcore_version,
            mod_management::delete_northstar_mod,
            util::get_server_player_count,
            util::kill_northstar,
            mod_management::delete_thunderstore_mod,
            platform_specific::install_northstar_proton_wrapper,
            platform_specific::uninstall_northstar_proton_wrapper,
            platform_specific::get_local_northstar_proton_wrapper_version,
            util::open_repair_window,
            thunderstore::query_thunderstore_packages_api,
            github::get_list_of_tags,
            github::compare_tags,
            github::pull_requests::get_pull_requests_wrapper,
            github::pull_requests::apply_launcher_pr,
            github::pull_requests::apply_mods_pr,
            github::pull_requests::get_launcher_download_link,
            util::close_application,
            development::install_git_main,
            get_available_northstar_versions,
            northstar::profile::fetch_profiles,
            northstar::profile::validate_profile,
            northstar::profile::delete_profile,
        ])
        .run(tauri::generate_context!())
    {
        Ok(()) => (),
        Err(err) => {
            // Failed to launch system native web view

            // Log error on Linux
            #[cfg(not(target_os = "windows"))]
            {
                log::error!("{err}");
            }

            // On Windows we can show an error window using Windows API to show how to install WebView2
            #[cfg(target_os = "windows")]
            {
                log::error!("WebView2 not installed: {err}");
                let dialog = MessageDialogBuilder::new(
                    "WebView2 not found",
                    "FlightCore requires WebView2 to run.\n\nClick OK to open installation instructions."
                )
                .kind(MessageDialogKind::Error)
                .buttons(MessageDialogButtons::Ok);

                if dialog.show() {
                    // Open the installation instructions URL in the user's default web browser
                    open::that("https://github.com/R2NorthstarTools/FlightCore/blob/main/docs/TROUBLESHOOTING.md#flightcore-wont-launch").unwrap();
                }
            }
        }
    };
}

/// Helps with converting release candidate numbers which are different on Thunderstore
/// due to restrictions imposed by the platform
pub fn convert_release_candidate_number(version_number: String) -> String {
    // This simply converts `-rc` to `0`
    // Works as intended for RCs < 10, e.g.  `v1.9.2-rc1`  -> `v1.9.201`
    // Doesn't work for larger numbers, e.g. `v1.9.2-rc11` -> `v1.9.2011` (should be `v1.9.211`)
    version_number.replace("-rc", "0").replace("00", "")
}

/// Checks if installed Northstar version is up-to-date
/// false -> Northstar install is up-to-date
/// true  -> Northstar install is outdated
#[tauri::command]
async fn check_is_northstar_outdated(
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

    let version_number = match northstar::get_northstar_version_number(game_install) {
        Ok(version_number) => version_number,
        Err(err) => {
            log::warn!("{}", err);
            // If we fail to get new version just assume we are up-to-date
            return Err(err);
        }
    };

    // Release candidate version numbers are different between `mods.json` and Thunderstore
    let version_number = convert_release_candidate_number(version_number);

    if version_number != nmod.latest {
        log::info!("Installed Northstar version outdated");
        Ok(true)
    } else {
        log::info!("Installed Northstar version up-to-date");
        Ok(false)
    }
}

/// Installs the specified mod
#[tauri::command]
async fn install_mod_wrapper(
    game_install: GameInstall,
    thunderstore_mod_string: String,
) -> Result<(), String> {
    match mod_management::fc_download_mod_and_install(&game_install, &thunderstore_mod_string).await
    {
        Ok(()) => (),
        Err(err) => {
            log::warn!("{err}");
            return Err(err);
        }
    };
    match repair_and_verify::clean_up_download_folder(&game_install, false) {
        Ok(()) => Ok(()),
        Err(err) => {
            log::info!("Failed to delete download folder due to {}", err);
            // Failure to delete download folder is not an error in mod install
            // As such ignore. User can still force delete if need be
            Ok(())
        }
    }
}

/// Installs the specified mod
#[tauri::command]
async fn clean_up_download_folder_wrapper(
    game_install: GameInstall,
    force: bool,
) -> Result<(), String> {
    match repair_and_verify::clean_up_download_folder(&game_install, force) {
        Ok(()) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

/// Gets list of available Northstar versions from Thunderstore
#[tauri::command]
async fn get_available_northstar_versions() -> Result<Vec<NorthstarThunderstoreReleaseWrapper>, ()>
{
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
        let a_ver = Version::parse(&a.value.version).unwrap();
        let b_ver = Version::parse(&b.value.version).unwrap();
        b_ver.partial_cmp(&a_ver).unwrap() // Sort newest first
    });

    Ok(releases)
}

use anyhow::Result;

/// Defines how Titanfall2 was installed (Steam, Origin, ...)
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum InstallType {
    STEAM,
    ORIGIN,
    EAPLAY,
    UNKNOWN,
}

/// Object holding information of the Titanfall2 install, including
/// - Install path
/// - Active profile
/// - Type of installation (Steam, Origin, ...)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInstall {
    pub game_path: String,
    pub profile: String,
    pub install_type: InstallType,
}

/// Object holding various information about a Northstar mod
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct NorthstarMod {
    pub name: String,
    pub version: Option<String>,
    pub thunderstore_mod_string: Option<String>,
    pub enabled: bool,
    pub directory: String,
}
