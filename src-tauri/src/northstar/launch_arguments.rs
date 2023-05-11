#[tauri::command]
/// This method loads arguments from the ns_startup_args.txt Northstar launch
/// arguments files, filtering out eventual argument duplicates.
/// If this file does not exist, this will return an empty array.
pub fn get_launch_arguments(game_path: &str) -> Result<Vec<String>, ()> {
    let launch_args_path = format!("{}/ns_startup_args.txt", game_path);
    if !std::path::Path::new(&launch_args_path).exists() {
        return Ok(vec![]);
    }

    let data = match std::fs::read_to_string(launch_args_path.clone()) {
        Ok(content) => content,
        Err(_) => {
            return Ok(vec![]);
        }
    };

    let mut arguments = data.split_whitespace()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>();
    arguments.sort_unstable();
    arguments.dedup();
    Ok(arguments)
}

#[tauri::command]
/// This method puts an array of arguments into the ns_startup_args.txt Northstar
/// launch arguments files.
/// If the ns_startup_args.txt file does not exist, this will create it.
pub fn set_launch_arguments(game_path: &str, arguments: Vec<String>) -> Result<(), String> {
    let launch_args_path = format!("{}/ns_startup_args.txt", game_path);
    let write_result = std::fs::write(launch_args_path.clone(), arguments.join(" "));
    
    if write_result.is_ok() {
        log::info!("Launch arguments updated.");
        return Ok(());
    } else {
        return Err("Failed to save launch arguments.".to_string());
    }
}
