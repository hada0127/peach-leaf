use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::{Manager, Emitter};
use crate::models::{AppState, StickerData};
use crate::state::{load_app_state, save_app_state, get_notes_dir, ensure_notes_dir};

// Store for window metadata (background colors, modes, etc.)
pub static WINDOW_METADATA: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, StickerData>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[tauri::command]
pub async fn save_window_state(app: tauri::AppHandle) -> Result<(), String> {
    save_window_state_impl(&app)
}

#[tauri::command]
pub async fn get_saved_state() -> Result<AppState, String> {
    load_app_state()
}

#[tauri::command]
pub async fn get_window_data(window_label: String) -> Result<Option<StickerData>, String> {
    let state = load_app_state()?;
    let window_data = state.windows.into_iter().find(|w| w.id == window_label);
    println!("get_window_data called for '{}': found = {}", window_label, window_data.is_some());
    Ok(window_data)
}

#[tauri::command]
pub async fn on_window_focus(window_label: String) -> Result<(), String> {
    use crate::menu::update_font_menu_checks;

    println!("on_window_focus called for '{}'", window_label);

    // Get window metadata to update menu checks
    let metadata = WINDOW_METADATA.lock().unwrap();
    if let Some(data) = metadata.get(&window_label) {
        println!("Updating font menu checks for window {} with font_size={}", window_label, data.font_size);
        update_font_menu_checks(data.font_size);
    } else {
        // Default to medium (14px) if no metadata
        update_font_menu_checks(14);
    }

    Ok(())
}

#[tauri::command]
pub async fn update_window_metadata(
    window_label: String,
    background_color: Option<String>,
    mode: Option<String>,
    font_size: Option<u32>
) -> Result<(), String> {
    println!("update_window_metadata called: window={}, bg_color={:?}, mode={:?}, font_size={:?}",
             window_label, background_color, mode, font_size);

    let mut metadata = WINDOW_METADATA.lock().unwrap();

    if let Some(data) = metadata.get_mut(&window_label) {
        if let Some(bg_color) = background_color {
            println!("Updating background_color for {}: {}", window_label, bg_color);
            data.background_color = bg_color;
        }
        if let Some(new_mode) = mode {
            println!("Updating mode for {}: {}", window_label, new_mode);
            data.mode = new_mode.clone();
        }
        if let Some(new_font_size) = font_size {
            println!("Updating font_size for {}: {}", window_label, new_font_size);
            data.font_size = new_font_size;
        }
        println!("Updated metadata for {}: color={}, mode={}, font_size={}",
                 window_label, data.background_color, data.mode, data.font_size);
    } else {
        // If metadata doesn't exist yet, create it with minimal info
        let notes_dir = ensure_notes_dir()?;
        let file_path = notes_dir.join(format!("{}.md", window_label));

        metadata.insert(window_label.clone(), StickerData {
            id: window_label,
            file_path: file_path.to_string_lossy().to_string(),
            x: 0,
            y: 0,
            width: 400,
            height: 300,
            background_color: background_color.unwrap_or_else(|| "#FEFCE8".to_string()),
            text_color: "#333333".to_string(),
            mode: mode.unwrap_or_else(|| "edit".to_string()),
            font_size: font_size.unwrap_or(14),
            monitor_name: None,
            monitor_position: None,
            monitor_size: None,
        });
    }

    Ok(())
}

#[tauri::command]
pub async fn create_sticker_window(
    app: tauri::AppHandle,
    sticker_data: StickerData,
) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;
    use tauri::WebviewUrl;

    let window = WebviewWindowBuilder::new(
        &app,
        &sticker_data.id,
        WebviewUrl::default(),
    )
    .title("PeachLeaf")
    .inner_size(sticker_data.width as f64, sticker_data.height as f64)
    .position(sticker_data.x as f64, sticker_data.y as f64)
    .decorations(false)
    .resizable(true)
    .build()
    .map_err(|e| e.to_string())?;

    // Send sticker data to the window
    window.emit("init-sticker", sticker_data)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn save_window_state_impl(app: &tauri::AppHandle) -> Result<(), String> {
    println!("Saving window state...");
    let mut windows_data = Vec::new();

    // Get metadata
    let metadata = WINDOW_METADATA.lock().unwrap();

    // Sort windows by label to ensure consistent order
    let webview_windows = app.webview_windows();
    let mut windows: Vec<_> = webview_windows.iter().collect();
    windows.sort_by_key(|(label, _)| label.as_str());

    for (label, window) in windows {
        // Skip color picker window
        if label.as_str() == "color-picker" {
            continue;
        }

        // Get window position and size
        let position = window.outer_position().map_err(|e| e.to_string())?;
        let size = window.outer_size().map_err(|e| e.to_string())?;

        // Get window scale factor
        let scale_factor = window.scale_factor().map_err(|e| e.to_string())?;

        // Convert to logical pixels
        let x = (position.x as f64 / scale_factor) as i32;
        let y = (position.y as f64 / scale_factor) as i32;
        let width = (size.width as f64 / scale_factor) as u32;
        let height = (size.height as f64 / scale_factor) as u32;

        // Get background color, mode, and font_size from metadata, or use defaults
        let background_color = metadata
            .get(label.as_str())
            .map(|data| data.background_color.clone())
            .unwrap_or_else(|| "#FEFCE8".to_string());

        let mode = metadata
            .get(label.as_str())
            .map(|data| data.mode.clone())
            .unwrap_or_else(|| "edit".to_string());

        let font_size = metadata
            .get(label.as_str())
            .map(|data| data.font_size)
            .unwrap_or(14);

        // Get monitor information
        let (monitor_name, monitor_position, monitor_size) = match window.current_monitor() {
            Ok(Some(monitor)) => {
                let name = monitor.name().map(|s| s.to_string());
                let pos = monitor.position();
                let size = monitor.size();
                (
                    name,
                    Some((pos.x, pos.y)),
                    Some((size.width, size.height))
                )
            }
            _ => (None, None, None)
        };

        // Create file path for this window using permanent directory
        let notes_dir = get_notes_dir();
        let file_path = notes_dir.join(format!("{}.md", label));
        let file_path_str = file_path.to_string_lossy().to_string();

        // Debug: check if metadata exists for this window
        if metadata.contains_key(label.as_str()) {
            println!("Found metadata for window {}: color={}, mode={}, font_size={}", label, background_color, mode, font_size);
        } else {
            println!("No metadata found for window {}, using defaults", label);
        }

        let sticker_data = StickerData {
            id: label.to_string(),
            file_path: file_path_str.clone(),
            x,
            y,
            width,
            height,
            background_color: background_color.clone(),
            text_color: "#333333".to_string(),
            mode: mode.clone(),
            font_size,
            monitor_name: monitor_name.clone(),
            monitor_position,
            monitor_size,
        };

        windows_data.push(sticker_data);
        println!("Saved window {}: position=({}, {}), size=({}x{}), color={}, mode={}, font_size={}, path={}",
                 label, x, y, width, height, background_color, mode, font_size, file_path_str);
    }

    // Sort windows_data by id to ensure consistent order in state.json
    windows_data.sort_by(|a, b| a.id.cmp(&b.id));

    save_app_state(windows_data)?;
    println!("Window state saved successfully");
    Ok(())
}
