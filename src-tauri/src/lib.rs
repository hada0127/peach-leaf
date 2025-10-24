use tauri::{Emitter, Manager, menu::{Menu, MenuBuilder, SubmenuBuilder, MenuItemBuilder}};
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StickerData {
    id: String,
    file_path: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    background_color: String,
    text_color: String,
    mode: String,
}

#[tauri::command]
async fn read_file(file_path: String) -> Result<String, String> {
    fs::read_to_string(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_file(file_path: String, content: String) -> Result<(), String> {
    fs::write(&file_path, content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn select_file(_app: tauri::AppHandle) -> Result<Option<String>, String> {
    // For now, return None - file dialog will be added later
    // In Tauri 2.x, file dialog is a separate plugin
    Ok(None)
}

#[tauri::command]
async fn create_sticker_window(
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

#[tauri::command]
async fn open_color_picker(
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
async fn close_color_picker(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(picker) = app.get_webview_window("color-picker") {
        picker.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn apply_color(
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::RunEvent;

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            read_file,
            write_file,
            select_file,
            create_sticker_window,
            open_color_picker,
            close_color_picker,
            apply_color
        ])
        .setup(|app| {
            // Create menu
            let menu = create_menu(app)?;
            app.set_menu(menu)?;

            // Handle menu events with debouncing
            let last_menu_event: Arc<Mutex<Option<(String, Instant)>>> = Arc::new(Mutex::new(None));

            app.on_menu_event(move |app, event| {
                let menu_id = event.id().as_ref();

                // Debounce: ignore events within 300ms
                {
                    let mut last_event = last_menu_event.lock().unwrap();
                    let now = Instant::now();

                    if let Some((last_id, last_time)) = last_event.as_ref() {
                        if last_id == menu_id && now.duration_since(*last_time) < Duration::from_millis(300) {
                            println!("Ignoring duplicate menu event: {}", menu_id);
                            return;
                        }
                    }

                    *last_event = Some((menu_id.to_string(), now));
                }

                println!("Menu clicked: {}", menu_id);

                // Handle new_note in backend to avoid duplicate creation
                if menu_id == "new_note" {
                    println!("Handling new_note in backend");
                    create_new_note_backend(app);
                    return;
                }

                // Handle close_note: emit ONLY to focused window
                if menu_id == "close_note" {
                    println!("Handling close_note in backend");
                    if let Some(focused_window) = app.webview_windows().values().find(|w| {
                        w.is_focused().unwrap_or(false)
                    }) {
                        println!("Emitting close_note to focused window: {}", focused_window.label());
                        let _ = focused_window.emit(&format!("close_note_{}", focused_window.label()), ());
                    }
                    return;
                }

                // Handle open_color_picker: open for focused window only
                if menu_id == "open_color_picker" {
                    println!("Handling open_color_picker in backend");
                    if let Some(focused_window) = app.webview_windows().values().find(|w| {
                        w.is_focused().unwrap_or(false)
                    }) {
                        let window_label = focused_window.label().to_string();
                        println!("Opening color picker for focused window: {}", window_label);

                        // Emit window-specific event
                        let _ = focused_window.emit(&format!("open_color_picker_{}", window_label), ());
                    }
                    return;
                }

                // Emit event to the focused window (for other menu items)
                if let Some(focused_window) = app.webview_windows().values().find(|w| {
                    w.is_focused().unwrap_or(false)
                }) {
                    println!("Emitting to focused window: {}", focused_window.label());
                    let _ = focused_window.emit("menu", menu_id);
                } else {
                    // Fallback to main window if no window is focused
                    if let Some(window) = app.get_webview_window("main") {
                        println!("No focused window, emitting to main");
                        let _ = window.emit("menu", menu_id);
                    }
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            // Prevent app from exiting when all windows are closed
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}

fn create_menu(app: &tauri::App) -> Result<Menu<tauri::Wry>, tauri::Error> {
    use tauri::menu::PredefinedMenuItem;

    // App Menu (PeachLeaf) - macOS only
    #[cfg(target_os = "macos")]
    let app_menu = SubmenuBuilder::new(app, "PeachLeaf")
        .item(&PredefinedMenuItem::hide(app, None)?)
        .item(&PredefinedMenuItem::quit(app, None)?)
        .build()?;

    // File Menu
    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&MenuItemBuilder::new("New Note").id("new_note").accelerator("CmdOrCtrl+N").build(app)?)
        .item(&MenuItemBuilder::new("Close Note").id("close_note").accelerator("CmdOrCtrl+W").build(app)?)
        .build()?;

    // Edit Menu
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&MenuItemBuilder::new("Undo").id("undo").accelerator("CmdOrCtrl+Z").build(app)?)
        .item(&MenuItemBuilder::new("Redo").id("redo").accelerator("CmdOrCtrl+Shift+Z").build(app)?)
        .separator()
        .item(&MenuItemBuilder::new("Cut").id("cut").accelerator("CmdOrCtrl+X").build(app)?)
        .item(&MenuItemBuilder::new("Copy").id("copy").accelerator("CmdOrCtrl+C").build(app)?)
        .item(&MenuItemBuilder::new("Paste").id("paste").accelerator("CmdOrCtrl+V").build(app)?)
        .build()?;

    // Font Menu
    let font_menu = SubmenuBuilder::new(app, "Font")
        .item(&MenuItemBuilder::new("Default Font").id("font_default").build(app)?)
        .separator()
        .item(&MenuItemBuilder::new("Small (12px)").id("font_small").build(app)?)
        .item(&MenuItemBuilder::new("Medium (14px)").id("font_medium").build(app)?)
        .item(&MenuItemBuilder::new("Large (16px)").id("font_large").build(app)?)
        .item(&MenuItemBuilder::new("Extra Large (18px)").id("font_xlarge").build(app)?)
        .build()?;

    // Color Menu - single item that opens picker
    let color_menu = SubmenuBuilder::new(app, "Color")
        .item(&MenuItemBuilder::new("Choose Color...").id("open_color_picker").build(app)?)
        .build()?;

    // Window Menu
    let window_menu = SubmenuBuilder::new(app, "Window")
        .item(&MenuItemBuilder::new("Minimize").id("minimize").build(app)?)
        .item(&MenuItemBuilder::new("Zoom").id("zoom").build(app)?)
        .build()?;

    // Help Menu
    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&MenuItemBuilder::new("About PeachLeaf").id("about").build(app)?)
        .build()?;

    // Build the menu bar
    #[cfg(target_os = "macos")]
    let menu = MenuBuilder::new(app)
        .item(&app_menu)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&font_menu)
        .item(&color_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()?;

    #[cfg(not(target_os = "macos"))]
    let menu = MenuBuilder::new(app)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&font_menu)
        .item(&color_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()?;

    Ok(menu)
}

fn create_main_window(app: &tauri::AppHandle) -> Result<(), tauri::Error> {
    use tauri::WebviewWindowBuilder;
    use tauri::WebviewUrl;

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

fn create_new_note_backend(app: &tauri::AppHandle) {
    use tauri::WebviewWindowBuilder;
    use tauri::WebviewUrl;
    use std::time::{SystemTime, UNIX_EPOCH};

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
    let temp_file_path = format!("/tmp/{}.md", new_id);

    // Random offset for window position
    let random_offset = (timestamp % 100) as i32 + 50;

    // Create sticker data
    let sticker_data = StickerData {
        id: new_id.clone(),
        file_path: temp_file_path.clone(),
        x: 150 + random_offset,
        y: 150 + random_offset,
        width: 400,
        height: 300,
        background_color: "#FEFCE8".to_string(),
        text_color: "#333333".to_string(),
        mode: "edit".to_string(),
    };

    // Write empty file
    if let Err(e) = fs::write(&temp_file_path, "") {
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
        }
        Err(e) => {
            eprintln!("Failed to create window: {}", e);
        }
    }
}
