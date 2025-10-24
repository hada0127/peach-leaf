use tauri::{Manager, Emitter, WebviewWindowBuilder, WebviewUrl};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::StickerData;
use crate::state::ensure_notes_dir;
use crate::commands::window::{WINDOW_METADATA, save_window_state_impl};

pub fn create_main_window(app: &tauri::AppHandle) -> Result<(), tauri::Error> {
    let _window = WebviewWindowBuilder::new(
        app,
        "main",
        WebviewUrl::default(),
    )
    .title("PeachLeaf")
    .inner_size(400.0, 300.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .always_on_top(false)
    .build()?;

    Ok(())
}

pub fn restore_window(app: &tauri::AppHandle, sticker_data: StickerData) {
    println!("Restoring window: {} at ({}, {})", sticker_data.id, sticker_data.x, sticker_data.y);

    // Calculate absolute position based on saved monitor info
    let (abs_x, abs_y) = if let (Some(saved_monitor_pos), Some(saved_monitor_size)) =
        (sticker_data.monitor_position, sticker_data.monitor_size) {

        println!("Saved monitor info: name={:?}, pos=({}, {}), size=({}, {})",
                 sticker_data.monitor_name, saved_monitor_pos.0, saved_monitor_pos.1,
                 saved_monitor_size.0, saved_monitor_size.1);

        // Try to find a matching monitor
        let available_monitors = app.available_monitors().unwrap_or_default();
        let target_monitor = available_monitors.iter().find(|m| {
            // First try to match by name if available
            if let Some(ref saved_name) = sticker_data.monitor_name {
                if let Some(current_name) = m.name() {
                    if current_name == saved_name {
                        return true;
                    }
                }
            }
            // Then try to match by position and size
            let pos = m.position();
            let size = m.size();
            pos.x == saved_monitor_pos.0 && pos.y == saved_monitor_pos.1 &&
            size.width == saved_monitor_size.0 && size.height == saved_monitor_size.1
        });

        if let Some(monitor) = target_monitor {
            let monitor_pos = monitor.position();
            println!("Found matching monitor at ({}, {})", monitor_pos.x, monitor_pos.y);
            // Use absolute position (monitor position + relative window position)
            (sticker_data.x, sticker_data.y)
        } else {
            println!("No matching monitor found, using saved position as-is");
            (sticker_data.x, sticker_data.y)
        }
    } else {
        println!("No monitor info saved, using position as-is");
        (sticker_data.x, sticker_data.y)
    };

    // Populate WINDOW_METADATA with the restored window's data
    {
        let mut metadata = WINDOW_METADATA.lock().unwrap();
        metadata.insert(sticker_data.id.clone(), sticker_data.clone());
        println!("Populated metadata for window {}: color={}, mode={}", sticker_data.id, sticker_data.background_color, sticker_data.mode);
    }

    match WebviewWindowBuilder::new(
        app,
        &sticker_data.id,
        WebviewUrl::default(),
    )
    .title("PeachLeaf")
    .inner_size(sticker_data.width as f64, sticker_data.height as f64)
    .position(abs_x as f64, abs_y as f64)
    .decorations(false)
    .resizable(true)
    .transparent(true)
    .always_on_top(false)
    .build() {
        Ok(window) => {
            // Send sticker data to the window after it's created
            if let Err(e) = window.emit("init-sticker", &sticker_data) {
                eprintln!("Failed to emit init-sticker event: {}", e);
            }
            println!("Window {} restored successfully", sticker_data.id);
        }
        Err(e) => {
            eprintln!("Failed to restore window {}: {}", sticker_data.id, e);
        }
    }
}

pub fn create_new_note_backend(app: &tauri::AppHandle) {
    // If no windows exist, create main window
    if app.webview_windows().is_empty() {
        println!("No windows exist, creating main window");
        if let Err(e) = create_main_window(app) {
            eprintln!("Failed to create main window: {}", e);
        }
        return;
    }

    // Generate unique ID
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let new_id = format!("note-{}", timestamp);

    // Use permanent directory for notes
    let notes_dir = match ensure_notes_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to create notes directory: {}", e);
            return;
        }
    };
    let file_path = notes_dir.join(format!("{}.md", new_id));
    let file_path_str = file_path.to_string_lossy().to_string();

    // Random offset for window position
    let random_offset = (timestamp % 100) as i32 + 50;

    // Create sticker data
    let sticker_data = StickerData {
        id: new_id.clone(),
        file_path: file_path_str.clone(),
        x: 150 + random_offset,
        y: 150 + random_offset,
        width: 400,
        height: 300,
        background_color: "#FEFCE8".to_string(),
        text_color: "#333333".to_string(),
        mode: "edit".to_string(),
        monitor_name: None,
        monitor_position: None,
        monitor_size: None,
    };

    // Write empty file
    if let Err(e) = fs::write(&file_path, "") {
        eprintln!("Failed to create file: {}", e);
        return;
    }

    // Create window
    match WebviewWindowBuilder::new(
        app,
        &new_id,
        WebviewUrl::default(),
    )
    .title("PeachLeaf")
    .inner_size(400.0, 300.0)
    .position((150 + random_offset) as f64, (150 + random_offset) as f64)
    .decorations(false)
    .resizable(true)
    .build() {
        Ok(window) => {
            // Send sticker data to the window
            if let Err(e) = window.emit("init-sticker", sticker_data) {
                eprintln!("Failed to emit init-sticker: {}", e);
            }

            // Save window state immediately after creating new window
            if let Err(e) = save_window_state_impl(app) {
                eprintln!("Failed to save window state after creating new note: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to create window: {}", e);
        }
    }
}
