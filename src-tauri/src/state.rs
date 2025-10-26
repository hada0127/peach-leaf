use std::fs;
use std::path::PathBuf;
use crate::models::{AppState, StickerData};

pub fn get_state_file_path() -> PathBuf {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home_dir).join(".peach-leaf").join("state.json")
}

pub fn get_notes_dir() -> PathBuf {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home_dir).join(".peach-leaf").join("notes")
}

pub fn ensure_notes_dir() -> Result<PathBuf, String> {
    let notes_dir = get_notes_dir();
    fs::create_dir_all(&notes_dir).map_err(|e| e.to_string())?;
    Ok(notes_dir)
}

pub fn save_app_state(windows: Vec<StickerData>) -> Result<(), String> {
    let state = AppState { windows };
    let json = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    let state_path = get_state_file_path();

    // Ensure .peach-leaf directory exists
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(&state_path, json).map_err(|e| e.to_string())?;
    println!("App state saved to: {:?}", state_path);
    Ok(())
}

pub fn load_app_state() -> Result<AppState, String> {
    let state_path = get_state_file_path();

    if !state_path.exists() {
        println!("No saved state found at {:?}", state_path);
        return Ok(AppState { windows: vec![] });
    }

    let json = fs::read_to_string(&state_path).map_err(|e| {
        eprintln!("Failed to read state file: {}", e);
        e.to_string()
    })?;

    let state: AppState = serde_json::from_str(&json).map_err(|e| {
        eprintln!("Failed to parse state JSON: {}", e);
        e.to_string()
    })?;

    println!("App state loaded from: {:?} ({} windows)", state_path, state.windows.len());
    Ok(state)
}

pub fn cleanup_orphaned_notes(state: &AppState) -> Result<(), String> {
    let notes_dir = get_notes_dir();

    // Get all note IDs from state.json
    let valid_ids: std::collections::HashSet<String> = state.windows
        .iter()
        .map(|w| w.id.clone())
        .collect();

    println!("Valid note IDs from state.json: {:?}", valid_ids);

    // Read all .md files in notes directory
    match fs::read_dir(&notes_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let note_id = file_stem.to_string();

                            // If this note ID is not in state.json, delete it
                            if !valid_ids.contains(&note_id) {
                                println!("Deleting orphaned note file: {:?}", path);
                                if let Err(e) = fs::remove_file(&path) {
                                    eprintln!("Failed to delete orphaned note {}: {}", note_id, e);
                                } else {
                                    println!("Successfully deleted orphaned note: {}", note_id);
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to read notes directory: {}", e);
            Err(e.to_string())
        }
    }
}
