use tauri::AppHandle;

#[tauri::command]
pub fn solve(_app: AppHandle) {}

#[tauri::command]
pub fn refine(_app: AppHandle) {}

#[tauri::command]
pub fn interrupt() {}
