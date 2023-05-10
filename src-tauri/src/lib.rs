use std::{cell::RefCell, env, fs, path::Path, time::Duration, time::Instant};

use anyhow::{Context, Result};

pub mod constants;
mod platform_specific;
#[cfg(target_os = "windows")]
use platform_specific::windows;

#[cfg(target_os = "linux")]
use platform_specific::linux;

use serde::{Deserialize, Serialize};
use sysinfo::SystemExt;
use tokio::time::sleep;
use ts_rs::TS;
use zip::ZipArchive;

use crate::constants::TITANFALL2_STEAM_ID;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InstallType {
    STEAM,
    ORIGIN,
    EAPLAY,
    UNKNOWN,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInstall {
    pub game_path: String,
    pub install_type: InstallType,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct NorthstarMod {
    pub name: String,
    pub version: Option<String>,
    pub thunderstore_mod_string: Option<String>,
    pub enabled: bool,
    pub directory: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NorthstarServer {
    #[serde(rename = "playerCount")]
    pub player_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum InstallState {
    DOWNLOADING,
    EXTRACTING,
    DONE,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
struct InstallProgress {
    current_downloaded: u64,
    total_size: u64,
    state: InstallState,
}

// I intend to add more linux related stuff to check here, so making a func
// for now tho it only checks `ldd --version`
// - salmon
#[cfg(target_os = "linux")]
pub fn linux_checks_librs() -> Result<(), String> {
    // Perform various checks in terms of Linux compatibility
    // Return early with error message if a check fails

    // check `ldd --version` to see if glibc is up to date for northstar proton
    let min_required_ldd_version = 2.33;
    let lddv = linux::check_glibc_v();
    if lddv < min_required_ldd_version {
        return Err(format!(
            "GLIBC is not version {} or greater",
            min_required_ldd_version
        ));
    };

    // All checks passed
    Ok(())
}

/// Attempts to find the game install location
pub fn find_game_install_location() -> Result<GameInstall, String> {
    // Attempt parsing Steam library directly
    match steamlocate::SteamDir::locate() {
        Some(mut steamdir) => {
            let titanfall2_steamid = TITANFALL2_STEAM_ID.parse().unwrap();
            match steamdir.app(&titanfall2_steamid) {
                Some(app) => {
                    // println!("{:#?}", app);
                    let game_install = GameInstall {
                        game_path: app.path.to_str().unwrap().to_string(),
                        install_type: InstallType::STEAM,
                    };
                    return Ok(game_install);
                }
                None => log::info!("Couldn't locate Titanfall2 Steam install"),
            }
        }
        None => log::info!("Couldn't locate Steam on this computer!"),
    }

    // (On Windows only) try parsing Windows registry for Origin install path
    #[cfg(target_os = "windows")]
    match windows::origin_install_location_detection() {
        Ok(game_path) => {
            let game_install = GameInstall {
                game_path,
                install_type: InstallType::ORIGIN,
            };
            return Ok(game_install);
        }
        Err(err) => {
            log::info!("{}", err);
        }
    };

    Err("Could not auto-detect game install location! Please enter it manually.".to_string())
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

/// Copied from `papa` source code and modified
///Extract N* zip file to target game path
// fn extract(ctx: &Ctx, zip_file: File, target: &Path) -> Result<()> {
fn extract(zip_file: std::fs::File, target: &std::path::Path) -> Result<()> {
    let mut archive = ZipArchive::new(&zip_file).context("Unable to open zip archive")?;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).unwrap();

        //This should work fine for N* because the dir structure *should* always be the same
        if f.enclosed_name().unwrap().starts_with("Northstar") {
            let out = target.join(
                f.enclosed_name()
                    .unwrap()
                    .strip_prefix("Northstar")
                    .unwrap(),
            );

            if (*f.name()).ends_with('/') {
                log::info!("Create directory {}", f.name());
                std::fs::create_dir_all(target.join(f.name()))
                    .context("Unable to create directory")?;
                continue;
            } else if let Some(p) = out.parent() {
                std::fs::create_dir_all(p).context("Unable to create directory")?;
            }

            let mut outfile = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&out)?;

            log::info!("Write file {}", out.display());

            std::io::copy(&mut f, &mut outfile).context("Unable to write to file")?;
        }
    }

    Ok(())
}

/// Copied from `papa` source code and modified
///Install N* from the provided mod
///
///Checks cache, else downloads the latest version
async fn do_install(
    window: tauri::Window,
    nmod: &thermite::model::ModVersion,
    game_path: &std::path::Path,
) -> Result<()> {
    let filename = format!("northstar-{}.zip", nmod.version);
    let download_directory = format!("{}/___flightcore-temp-download-dir/", game_path.display());

    std::fs::create_dir_all(download_directory.clone())?;

    let download_path = format!("{}/{}", download_directory, filename);
    log::info!("Download path: {download_path}");

    let last_emit = RefCell::new(Instant::now()); // Keep track of the last time a signal was emitted
    let nfile = thermite::core::manage::download_file_with_progress(
        &nmod.url,
        download_path,
        |delta, current, total| {
            if delta != 0 {
                // Only emit a signal once every 100ms
                // This way we don't bombard the frontend with events on fast download speeds
                let time_since_last_emit = Instant::now().duration_since(*last_emit.borrow());
                if time_since_last_emit >= Duration::from_millis(100) {
                    window
                        .emit(
                            "northstar-install-download-progress",
                            InstallProgress {
                                current_downloaded: current,
                                total_size: total,
                                state: InstallState::DOWNLOADING,
                            },
                        )
                        .unwrap();
                    *last_emit.borrow_mut() = Instant::now();
                }
            }
        },
    )
    .unwrap();

    window
        .emit(
            "northstar-install-download-progress",
            InstallProgress {
                current_downloaded: 0,
                total_size: 0,
                state: InstallState::EXTRACTING,
            },
        )
        .unwrap();

    log::info!("Extracting Northstar...");
    extract(nfile, game_path)?;

    // Delete old copy
    log::info!("Delete temp folder again");
    std::fs::remove_dir_all(download_directory).unwrap();

    log::info!("Done installing Northstar!");
    window
        .emit(
            "northstar-install-download-progress",
            InstallProgress {
                current_downloaded: 0,
                total_size: 0,
                state: InstallState::DONE,
            },
        )
        .unwrap();

    Ok(())
}

pub async fn install_northstar(
    window: tauri::Window,
    game_path: &str,
    northstar_package_name: String,
    version_number: Option<String>,
) -> Result<String, String> {
    let index = thermite::api::get_package_index().unwrap().to_vec();
    let nmod = index
        .iter()
        .find(|f| f.name.to_lowercase() == northstar_package_name.to_lowercase())
        .ok_or_else(|| panic!("Couldn't find Northstar on thunderstore???"))
        .unwrap();

    // Use passed version or latest if no version was passed
    let version = version_number.as_ref().unwrap_or(&nmod.latest);

    log::info!("Install path \"{}\"", game_path);

    match do_install(
        window,
        nmod.versions.get(version).unwrap(),
        std::path::Path::new(game_path),
    )
    .await
    {
        Ok(_) => (),
        Err(err) => {
            if game_path
                .to_lowercase()
                .contains(&r#"C:\Program Files\"#.to_lowercase())
            // default is `C:\Program Files\EA Games\Titanfall2`
            {
                return Err(
                    "Cannot install to default EA App install path, please move Titanfall2 to a different install location.".to_string(),
                );
            } else {
                return Err(err.to_string());
            }
        }
    }

    Ok(nmod.latest.clone())
}

/// Returns identifier of host OS FlightCore is running on
pub fn get_host_os() -> String {
    env::consts::OS.to_string()
}

/// Prepare Northstar and Launch through Steam using the Browser Protocol
pub fn launch_northstar_steam(
    game_install: &GameInstall,
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
                    Some(compat) => {
                        if !compat
                            .name
                            .clone()
                            .unwrap()
                            .to_ascii_lowercase()
                            .contains("northstarproton")
                        {
                            return Err(
                                "Titanfall2 was not configured to use NorthstarProton".to_string()
                            );
                        }
                    }
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
    if std::env::set_current_dir(game_install.game_path.clone()).is_err() {
        // We failed to get to Titanfall2 directory
        return Err("Couldn't access Titanfall2 directory".to_string());
    }

    let run_northstar = "run_northstar.txt";
    let run_northstar_bak = "run_northstar.txt.bak";

    if Path::new(run_northstar).exists() {
        // rename should ovewrite existing files
        fs::rename(run_northstar, run_northstar_bak).unwrap();
    }

    // Passing arguments gives users a prompt, so we use run_northstar.txt
    fs::write(run_northstar, b"1").unwrap();

    let retval = match open::that(format!("steam://run/{}/", TITANFALL2_STEAM_ID)) {
        Ok(()) => Ok("Started game".to_string()),
        Err(_err) => Err("Failed to launch Titanfall 2 via Steam".to_string()),
    };

    let is_err = retval.is_err();

    // Handle the rest in the backround
    tauri::async_runtime::spawn(async move {
        // Starting the EA app and Titanfall might take a good minute or three
        let mut wait_countdown = 60 * 3;
        while wait_countdown > 0 && !check_northstar_running() && !is_err {
            sleep(Duration::from_millis(1000)).await;
            wait_countdown -= 1;
        }

        // Northstar may be running, but it may not have loaded the file yet
        sleep(Duration::from_millis(2000)).await;

        // intentionally ignore Result
        let _ = fs::remove_file(run_northstar);

        if Path::new(run_northstar_bak).exists() {
            fs::rename(run_northstar_bak, run_northstar).unwrap();
        }
    });

    retval
}

pub fn check_origin_running() -> bool {
    let s = sysinfo::System::new_all();
    let x = s.processes_by_name("Origin.exe").next().is_some()
        || s.processes_by_name("EADesktop.exe").next().is_some();
    x
}

/// Checks if Northstar process is running
pub fn check_northstar_running() -> bool {
    let s = sysinfo::System::new_all();
    let x = s
        .processes_by_name("NorthstarLauncher.exe")
        .next()
        .is_some()
        || s.processes_by_name("Titanfall2.exe").next().is_some();
    x
}
