use tauri::Manager;

mod commands;

pub struct AppState;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::solve,
            commands::refine,
            commands::interrupt
        ])
        .run(tauri::generate_context!())
        .expect("Tauri application error");
}
