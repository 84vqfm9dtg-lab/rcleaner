pub mod cli;
pub mod core;

#[tauri::command]
async fn scan_targets_command() -> Result<core::ScanReport, String> {
    tauri::async_runtime::spawn_blocking(|| core::scan_targets(None))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn clean_targets_command(
    target_ids: Vec<String>,
    dry_run: bool,
    allow_readonly: bool,
) -> Result<core::CleanReport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        core::clean_targets(core::CleanOptions {
            target_ids,
            clean_all: false,
            dry_run,
            allow_readonly,
            root_override: None,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn reveal_path_command(path: String) -> Result<(), String> {
    core::reveal_in_finder(path.into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            scan_targets_command,
            clean_targets_command,
            reveal_path_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
