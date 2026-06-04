use tauri::Manager;

use crate::types::ActiveOperationState;

mod commands;
mod save;
mod types;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(ActiveOperationState::default());
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::geneva_bank_holidays,
            commands::interrupt,
            commands::orchestrate,
            commands::refine,
            commands::solve,
            commands::statistics,
            commands::validate,
            save::load_project,
            save::save_project,
        ])
        .run(tauri::generate_context!())
        .expect("tauri application error");
}
