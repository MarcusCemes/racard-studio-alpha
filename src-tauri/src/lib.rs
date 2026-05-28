use tauri::Manager;

mod commands;
mod types;

pub struct AppState;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::geneva_bank_holidays,
            commands::interrupt,
            commands::orchestrate,
            commands::refine,
            commands::solve,
            commands::statistics,
            commands::validate,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri application error");
}
