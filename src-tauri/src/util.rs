//! This module contains various utility/helper functions that do not fit into any other module

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

use crate::constants::{APP_USER_AGENT, MASTER_SERVER_URL, SERVER_BROWSER_ENDPOINT};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NorthstarServer {
    #[serde(rename = "playerCount")]
    pub player_count: i32,
}

/// This function's only use is to force a `panic!()`
// This must NOT be async to ensure crashing whole application.
#[tauri::command]
pub fn force_panic() {
    panic!("Force panicked!");
}

/// Returns true if built in debug mode
#[tauri::command]
pub async fn is_debug_mode() -> bool {
    cfg!(debug_assertions)
}

/// Returns the current version number as a string
#[tauri::command]
pub async fn get_flightcore_version_number() -> String {
    let version = env!("CARGO_PKG_VERSION");
    if cfg!(debug_assertions) {
        // Debugging enabled
        format!("v{} (debug mode)", version)
    } else {
        // Debugging disabled
        format!("v{}", version)
    }
}

/// Spawns repair window
#[tauri::command]
pub async fn open_repair_window(handle: tauri::AppHandle) -> Result<(), String> {
    // Spawn new window
    let repair_window = match tauri::WindowBuilder::new(
        &handle,
        "RepairWindow",
        tauri::WindowUrl::App("/#/repair".into()),
    )
    .build()
    {
        Ok(res) => res,
        Err(err) => return Err(err.to_string()),
    };

    // Set window title
    match repair_window.set_title("FlightCore Repair Window") {
        Ok(()) => (),
        Err(err) => return Err(err.to_string()),
    };
    Ok(())
}

/// Closes all windows and exits application
#[tauri::command]
pub async fn close_application<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    app.exit(0); // Close application
    Ok(())
}

/// Fetches `/client/servers` endpoint from master server
async fn fetch_server_list() -> Result<String, anyhow::Error> {
    let url = format!("{MASTER_SERVER_URL}{SERVER_BROWSER_ENDPOINT}");
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
        .send()
        .await?
        .text()
        .await?;

    Ok(res)
}

/// Gets server and playercount from master server API
#[tauri::command]
pub async fn get_server_player_count() -> Result<(i32, usize), String> {
    let res = match fetch_server_list().await {
        Ok(res) => res,
        Err(err) => return Err(err.to_string()),
    };

    let ns_servers: Vec<NorthstarServer> =
        serde_json::from_str(&res).expect("JSON was not well-formatted");

    // Get server count
    let server_count = ns_servers.len();

    // Sum up player count
    let total_player_count: i32 = ns_servers.iter().map(|server| server.player_count).sum();

    log::info!("total_player_count: {}", total_player_count);
    log::info!("server_count:       {}", server_count);

    Ok((total_player_count, server_count))
}

#[tauri::command]
pub async fn kill_northstar() -> Result<(), String> {
    if !check_northstar_running() {
        return Err("Northstar is not running".to_string());
    }

    let s = sysinfo::System::new_all();

    for process in s.processes_by_exact_name("Titanfall2.exe") {
        log::info!("Killing Process {}", process.pid());
        process.kill();
    }

    for process in s.processes_by_exact_name("NorthstarLauncher.exe") {
        log::info!("Killing Process {}", process.pid());
        process.kill();
    }

    Ok(())
}

/// Copied from `papa` source code and modified
///Extract N* zip file to target game path
// fn extract(ctx: &Ctx, zip_file: File, target: &Path) -> Result<()> {
pub fn extract(zip_file: std::fs::File, target: &std::path::Path) -> Result<()> {
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

pub fn check_ea_app_or_origin_running() -> bool {
    let s = sysinfo::System::new_all();
    let x = s.processes_by_name("Origin.exe").next().is_some()
        || s.processes_by_name("EADesktop.exe").next().is_some();
    x
}

/// Checks if Northstar process is running (ignores dedicated server)
pub fn check_northstar_running() -> bool {
    let mut s = sysinfo::System::new_all();
    s.refresh_all();
    for process in s.processes().values() {
        if (process.name().ends_with("Titanfall2.exe") || process.name().ends_with("Northstar.exe"))
            && !process.cmd().contains(&String::from("-dedicated"))
        {
            return true;
        }
    }
    false
}

/// Copies a folder and all its contents to a new location
pub fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Moves a folders file structure to a new location
/// Old folders are not removed
pub fn move_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            move_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            std::fs::remove_dir(entry.path())?;
        } else {
            std::fs::rename(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Helps with converting release candidate numbers which are different on Thunderstore
/// due to restrictions imposed by the platform
pub fn convert_release_candidate_number(version_number: String) -> String {
    let release_candidate_suffix = "-rc";

    if !version_number.contains(release_candidate_suffix) {
        // Not an release-candidate version number, nothing to do, return early
        return version_number;
    }

    // Version number is guaranteed to contain `-rc`
    let re = regex::Regex::new(r"(\d+)\.(\d+)\.(\d+)-rc(\d+)").unwrap();
    if let Some(captures) = re.captures(&version_number) {
        // Extract versions
        let major_version: u32 = captures[1].parse().unwrap();
        let minor_version: u32 = captures[2].parse().unwrap();
        let patch_version: u32 = captures[3].parse().unwrap();
        let release_candidate: u32 = captures[4].parse().unwrap();

        // Zero pad
        let padded_release_candidate = format!("{:02}", release_candidate);

        // Combine
        let combined_patch_version = format!("{}{}", patch_version, padded_release_candidate);

        // Strip leading zeroes
        let trimmed_combined_patch_version = combined_patch_version.trim_start_matches('0');

        // Combine all
        let version_number = format!(
            "{}.{}.{}",
            major_version, minor_version, trimmed_combined_patch_version
        );
        return version_number;
    }

    // We should never end up here
    panic!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_release_candidate() {
        let input = "1.2.3".to_string();
        let output = convert_release_candidate_number(input.clone());
        let expected_output = input;
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_basic_release_candidate_number_conversion() {
        let input = "1.2.3-rc4".to_string();
        let output = convert_release_candidate_number(input);
        let expected_output = "1.2.304";
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_leading_zero_release_candidate_number_conversion() {
        let input = "1.2.0-rc3".to_string();
        let output = convert_release_candidate_number(input);
        let expected_output = "1.2.3";
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_double_patch_digit_release_candidate_number_conversion() {
        // let input = "v1.2.34-rc5".to_string();
        // let output = convert_release_candidate_number(input);
        // let expected_output = "v1.2.3405";
        let input = "1.19.10-rc1".to_string();
        let output = convert_release_candidate_number(input);
        let expected_output = "1.19.1001";

        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_double_digit_release_candidate_number_conversion() {
        let input = "1.2.3-rc45".to_string();
        let output = convert_release_candidate_number(input);
        let expected_output = "1.2.345";

        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_double_digit_patch_and_rc_number_conversion() {
        let input = "1.2.34-rc56".to_string();
        let output = convert_release_candidate_number(input);
        let expected_output = "1.2.3456";

        assert_eq!(output, expected_output);
    }
}
