use tauri::{Manager, Emitter};

#[tauri::command]
pub async fn open_color_picker(
    app: tauri::AppHandle,
    parent_label: String,
    current_color: String,
) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;
    use tauri::WebviewUrl;

    // Get parent window position to place color picker below it
    let parent_window = app.get_webview_window(&parent_label)
        .ok_or("Parent window not found")?;

    // Calculate position below the Color menu
    // On macOS, the global menu bar is at the top of the screen
    // The Color menu is approximately at position: PeachLeaf(~60px) + File(~40px) + Edit(~40px) + Font(~40px) + Color(starts ~180px)
    // Y position should be just below the menu bar (macOS menu bar is ~25px tall)

    // Get screen size to ensure window is fully visible
    let monitor = parent_window.current_monitor().ok().flatten();
    let screen_width = monitor.map(|m| m.size().width).unwrap_or(1920);

    // Calculate X position for Color menu, ensuring it doesn't go off screen
    let picker_width = 252;
    let menu_x = 180; // Approximate Color menu position
    let x = if menu_x + picker_width > screen_width as i32 {
        (screen_width as i32 - picker_width - 10).max(10) // Keep 10px margin
    } else {
        menu_x
    };

    let y = 25; // Just below macOS menu bar (global menu bar height)

    // Close existing color picker if any
    if let Some(existing) = app.get_webview_window("color-picker") {
        let _ = existing.close();
    }

    // Create URL with query parameters
    use urlencoding::encode;
    let encoded_label = encode(&parent_label);
    let encoded_color = encode(&current_color);
    let url = format!("?parent_label={}&current_color={}", encoded_label, encoded_color);

    let _picker_window = WebviewWindowBuilder::new(
        &app,
        "color-picker",
        WebviewUrl::App(url.parse().unwrap()),
    )
    .title("Color Picker")
    .inner_size(252.0, 108.0) // 6*36 + 5*4 gaps + 2*8 padding = 252, 3*28 + 2*4 gaps + 2*8 padding = 108
    .position(x as f64, y as f64)
    .decorations(false)
    .resizable(false)
    .always_on_top(true)
    .focused(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn close_color_picker(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(picker) = app.get_webview_window("color-picker") {
        picker.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn apply_color(
    app: tauri::AppHandle,
    parent_label: String,
    color: String,
) -> Result<(), String> {
    println!("apply_color called: parent={}, color={}", parent_label, color);

    // Get parent window and emit window-specific color-selected event to it
    if let Some(parent_window) = app.get_webview_window(&parent_label) {
        #[derive(serde::Serialize, Clone)]
        struct ColorData {
            color: String,
        }

        let event_name = format!("color-selected-{}", parent_label);
        parent_window.emit(&event_name, ColorData { color: color.clone() })
            .map_err(|e| e.to_string())?;

        println!("Emitted {} event to window: {}", event_name, parent_label);
    } else {
        println!("Parent window not found: {}", parent_label);
        return Err(format!("Parent window not found: {}", parent_label));
    }

    // Close color picker
    if let Some(picker) = app.get_webview_window("color-picker") {
        picker.close().map_err(|e| e.to_string())?;
    }

    Ok(())
}
