use std::fs;
use crate::state::get_notes_dir;

#[tauri::command]
pub async fn read_file(file_path: String) -> Result<String, String> {
    println!("read_file called: {}", file_path);
    let result = fs::read_to_string(&file_path)
        .map_err(|e| e.to_string());
    match &result {
        Ok(content) => println!("read_file success: {} bytes", content.len()),
        Err(e) => println!("read_file error: {}", e),
    }
    result
}

#[tauri::command]
pub async fn write_file(file_path: String, content: String) -> Result<(), String> {
    fs::write(&file_path, content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn select_file(_app: tauri::AppHandle) -> Result<Option<String>, String> {
    // For now, return None - file dialog will be added later
    // In Tauri 2.x, file dialog is a separate plugin
    Ok(None)
}

#[tauri::command]
pub async fn delete_note_file(note_id: String) -> Result<(), String> {
    let notes_dir = get_notes_dir();
    let file_path = notes_dir.join(format!("{}.md", note_id));

    println!("Deleting note file: {:?}", file_path);

    if file_path.exists() {
        fs::remove_file(&file_path)
            .map_err(|e| e.to_string())?;
        println!("Successfully deleted note file: {}", note_id);
    } else {
        println!("Note file does not exist: {}", note_id);
    }

    Ok(())
}
