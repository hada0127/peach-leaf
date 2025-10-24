use tauri::{Manager, Emitter, WebviewWindowBuilder, WebviewUrl, Monitor};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::StickerData;
use crate::state::ensure_notes_dir;
use crate::commands::window::{WINDOW_METADATA, save_window_state_impl};

/// Validates if a window position is visible on any available monitor.
/// If not visible, returns a position on the primary monitor.
fn validate_window_position(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    monitors: &[Monitor]
) -> (i32, i32) {
    // Check if window is visible on any monitor
    let is_visible = monitors.iter().any(|monitor| {
        let pos = monitor.position();
        let size = monitor.size();

        // Window is visible if at least part of it is within the monitor bounds
        let window_right = x + width as i32;
        let window_bottom = y + height as i32;
        let monitor_right = pos.x + size.width as i32;
        let monitor_bottom = pos.y + size.height as i32;

        // Check for overlap
        x < monitor_right && window_right > pos.x &&
        y < monitor_bottom && window_bottom > pos.y
    });

    if is_visible {
        println!("Window position ({}, {}) is visible", x, y);
        return (x, y);
    }

    // Window is not visible - move to primary monitor
    println!("Window position ({}, {}) is NOT visible, relocating", x, y);

    if let Some(primary_monitor) = monitors.iter().find(|m| {
        let pos = m.position();
        pos.x == 0 && pos.y == 0
    }).or_else(|| monitors.first()) {
        let primary_pos = primary_monitor.position();
        let primary_size = primary_monitor.size();

        // Place window with some padding from top-left corner
        let new_x = primary_pos.x + 100;
        let new_y = primary_pos.y + 100;

        // Make sure window fits within monitor
        let max_x = primary_pos.x + primary_size.width as i32 - width as i32 - 50;
        let max_y = primary_pos.y + primary_size.height as i32 - height as i32 - 50;

        let final_x = new_x.min(max_x).max(primary_pos.x + 50);
        let final_y = new_y.min(max_y).max(primary_pos.y + 50);

        println!("Relocated to primary monitor at ({}, {})", final_x, final_y);
        (final_x, final_y)
    } else {
        println!("No monitors available, using default position");
        (100, 100)
    }
}

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

    // Get available monitors
    let available_monitors = app.available_monitors().unwrap_or_default();

    // Calculate absolute position based on saved monitor info
    let (abs_x, abs_y) = if let (Some(saved_monitor_pos), Some(saved_monitor_size)) =
        (sticker_data.monitor_position, sticker_data.monitor_size) {

        println!("Saved monitor info: name={:?}, pos=({}, {}), size=({}, {})",
                 sticker_data.monitor_name, saved_monitor_pos.0, saved_monitor_pos.1,
                 saved_monitor_size.0, saved_monitor_size.1);

        // Try to find a matching monitor
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
            println!("No matching monitor found, relocating to primary monitor");
            // Monitor not found - move to primary monitor
            if let Some(primary_monitor) = available_monitors.iter().find(|m| {
                // Primary monitor is typically at (0, 0) on macOS
                let pos = m.position();
                pos.x == 0 && pos.y == 0
            }).or_else(|| available_monitors.first()) {
                let primary_pos = primary_monitor.position();
                let primary_size = primary_monitor.size();

                // Calculate position to center the window on primary monitor
                let window_x = primary_pos.x + 100;
                let window_y = primary_pos.y + 100;

                // Make sure window is within monitor bounds
                let max_x = primary_pos.x + primary_size.width as i32 - sticker_data.width as i32 - 50;
                let max_y = primary_pos.y + primary_size.height as i32 - sticker_data.height as i32 - 50;

                let final_x = window_x.min(max_x).max(primary_pos.x + 50);
                let final_y = window_y.min(max_y).max(primary_pos.y + 50);

                println!("Relocated to primary monitor at ({}, {})", final_x, final_y);
                (final_x, final_y)
            } else {
                println!("No monitors available, using default position");
                (100, 100)
            }
        }
    } else {
        println!("No monitor info saved, checking if position is visible");
        // No saved monitor info - verify the window is within visible bounds
        validate_window_position(sticker_data.x, sticker_data.y,
                                 sticker_data.width, sticker_data.height,
                                 &available_monitors)
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
        font_size: 14,
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
