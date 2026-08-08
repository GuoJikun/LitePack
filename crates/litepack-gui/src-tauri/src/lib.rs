pub mod commands;
pub mod context_menu;
pub mod state;

use std::sync::Arc;

use state::{OperationRegistry, PendingExtract};

/// 解析 `--extract-here <path>` 启动参数。
fn pending_extract_arg() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--extract-here" {
            return args.next();
        }
    }
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pending = pending_extract_arg();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Arc::new(OperationRegistry::new()))
        .manage(PendingExtract(pending.into()))
        .invoke_handler(tauri::generate_handler![
            commands::compress_files,
            commands::extract_archive,
            commands::list_archive,
            commands::cancel_operation,
            commands::register_context_menu,
            commands::unregister_context_menu,
            commands::context_menu_status,
            commands::take_pending_extract,
            commands::exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
