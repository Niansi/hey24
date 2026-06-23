// TODO: Implement Tauri IPC commands (Task 8)

#[tauri::command]
pub fn detect_faces() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn align_photos() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn render_video() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn load_template() -> Result<(), String> {
    Ok(())
}
