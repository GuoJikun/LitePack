pub mod commands;
pub mod state;

use std::sync::Arc;

use state::OperationRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Arc::new(OperationRegistry::new()))
        .invoke_handler(tauri::generate_handler![
            commands::compress_files,
            commands::extract_archive,
            commands::list_archive,
            commands::cancel_operation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
