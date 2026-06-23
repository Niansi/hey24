pub mod engine;
mod error;
pub mod commands;

use commands::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            detector: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_faces,
            commands::align_photos,
            commands::render_video,
            commands::load_template,
            commands::check_system,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
