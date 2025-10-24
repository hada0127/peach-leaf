use tauri::{Emitter, Manager, menu::{Menu, MenuBuilder, SubmenuBuilder, MenuItemBuilder}};
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    windows: Vec<StickerData>,
}

fn get_state_file_path() -> PathBuf {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home_dir).join(".peach-leaf").join("state.json")
}

fn get_notes_dir() -> PathBuf {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home_dir).join(".peach-leaf").join("notes")
}

fn ensure_notes_dir() -> Result<PathBuf, String> {
    let notes_dir = get_notes_dir();
    fs::create_dir_all(&notes_dir).map_err(|e| e.to_string())?;
    Ok(notes_dir)
}

fn save_app_state(windows: Vec<StickerData>) -> Result<(), String> {
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

fn load_app_state() -> Result<AppState, String> {
    let state_path = get_state_file_path();
    if !state_path.exists() {
        println!("No saved state found");
        return Ok(AppState { windows: vec![] });
    }

    let json = fs::read_to_string(&state_path).map_err(|e| e.to_string())?;
    let state: AppState = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    println!("App state loaded from: {:?}", state_path);
    Ok(state)
}

#[tauri::command]
async fn read_file(file_path: String) -> Result<String, String> {
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
async fn save_window_state(app: tauri::AppHandle) -> Result<(), String> {
    println!("Saving window state...");
    let mut windows_data = Vec::new();

    // Get metadata
    let metadata = WINDOW_METADATA.lock().unwrap();

    for (label, window) in app.webview_windows().iter() {
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

        // Get background color from metadata, or use default
        let background_color = metadata
            .get(label.as_str())
            .map(|data| data.background_color.clone())
            .unwrap_or_else(|| "#FEFCE8".to_string());

        // Create file path for this window using permanent directory
        let notes_dir = get_notes_dir();
        let file_path = notes_dir.join(format!("{}.md", label));
        let file_path_str = file_path.to_string_lossy().to_string();

        // Debug: check if metadata exists for this window
        if metadata.contains_key(label.as_str()) {
            println!("Found metadata for window {}: color={}", label, background_color);
        } else {
            println!("No metadata found for window {}, using default color", label);
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
            mode: "edit".to_string(),
        };

        windows_data.push(sticker_data);
        println!("Saved window {}: position=({}, {}), size=({}x{}), color={}, path={}",
                 label, x, y, width, height, background_color, file_path_str);
    }

    save_app_state(windows_data)?;
    println!("Window state saved successfully");
    Ok(())
}

#[tauri::command]
async fn get_saved_state() -> Result<AppState, String> {
    load_app_state()
}

#[tauri::command]
async fn get_window_data(window_label: String) -> Result<Option<StickerData>, String> {
    let state = load_app_state()?;
    let window_data = state.windows.into_iter().find(|w| w.id == window_label);
    println!("get_window_data called for '{}': found = {}", window_label, window_data.is_some());
    Ok(window_data)
}

// Store for window metadata (background colors, etc.)
use std::collections::HashMap;
static WINDOW_METADATA: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, StickerData>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[tauri::command]
async fn update_window_metadata(window_label: String, background_color: String) -> Result<(), String> {
    let mut metadata = WINDOW_METADATA.lock().unwrap();

    if let Some(data) = metadata.get_mut(&window_label) {
        data.background_color = background_color;
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
            background_color,
            text_color: "#333333".to_string(),
            mode: "edit".to_string(),
        });
    }

    Ok(())
}

fn save_window_state_sync(app: &tauri::AppHandle) -> Result<(), String> {
    println!("Saving window state (sync)...");
    let mut windows_data = Vec::new();

    // Get metadata
    let metadata = WINDOW_METADATA.lock().unwrap();

    for (label, window) in app.webview_windows().iter() {
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

        // Get background color from metadata, or use default
        let background_color = metadata
            .get(label.as_str())
            .map(|data| data.background_color.clone())
            .unwrap_or_else(|| "#FEFCE8".to_string());

        // Create file path for this window using permanent directory
        let notes_dir = get_notes_dir();
        let file_path = notes_dir.join(format!("{}.md", label));
        let file_path_str = file_path.to_string_lossy().to_string();

        let sticker_data = StickerData {
            id: label.to_string(),
            file_path: file_path_str.clone(),
            x,
            y,
            width,
            height,
            background_color: background_color.clone(),
            text_color: "#333333".to_string(),
            mode: "edit".to_string(),
        };

        // Debug: check if metadata exists for this window
        if metadata.contains_key(label.as_str()) {
            println!("Found metadata for window {}: color={}", label, background_color);
        } else {
            println!("No metadata found for window {}, using default color", label);
        }

        windows_data.push(sticker_data);
        println!("Saved window {}: position=({}, {}), size=({}x{}), color={}, path={}",
                 label, x, y, width, height, background_color, file_path_str);
    }

    save_app_state(windows_data)?;
    println!("Window state saved successfully (sync)");
    Ok(())
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
            apply_color,
            save_window_state,
            get_saved_state,
            get_window_data,
            update_window_metadata
        ])
        .setup(|app| {
            // Create menu
            let menu = create_menu(app)?;
            app.set_menu(menu)?;

            // Restore saved windows
            let app_handle = app.app_handle();
            match load_app_state() {
                Ok(state) => {
                    if state.windows.is_empty() {
                        println!("No saved windows, creating default window");
                        if let Err(e) = create_main_window(&app_handle) {
                            eprintln!("Failed to create main window: {}", e);
                        }
                    } else {
                        println!("Restoring {} saved windows", state.windows.len());
                        for window_data in state.windows {
                            restore_window(&app_handle, window_data);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load saved state: {}", e);
                    if let Err(e) = create_main_window(&app_handle) {
                        eprintln!("Failed to create main window: {}", e);
                    }
                }
            }

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

                // Handle quit_app: just quit (state is saved on every change)
                if menu_id == "quit_app" {
                    println!("Handling quit_app in backend");

                    // Close all windows
                    let windows: Vec<_> = app.webview_windows().keys().map(|k| k.to_string()).collect();
                    for label in windows {
                        if let Some(window) = app.get_webview_window(&label) {
                            let _ = window.close();
                        }
                    }

                    // Exit the app
                    std::process::exit(0);
                }

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
        .item(&MenuItemBuilder::new("Quit PeachLeaf").id("quit_app").accelerator("CmdOrCtrl+Q").build(app)?)
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

fn restore_window(app: &tauri::AppHandle, sticker_data: StickerData) {
    use tauri::WebviewWindowBuilder;
    use tauri::WebviewUrl;

    println!("Restoring window: {} at ({}, {})", sticker_data.id, sticker_data.x, sticker_data.y);

    // Populate WINDOW_METADATA with the restored window's data
    {
        let mut metadata = WINDOW_METADATA.lock().unwrap();
        metadata.insert(sticker_data.id.clone(), sticker_data.clone());
        println!("Populated metadata for window {}: color={}", sticker_data.id, sticker_data.background_color);
    }

    match WebviewWindowBuilder::new(
        app,
        &sticker_data.id,
        WebviewUrl::default(),
    )
    .title("PeachLeaf")
    .inner_size(sticker_data.width as f64, sticker_data.height as f64)
    .position(sticker_data.x as f64, sticker_data.y as f64)
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
            if let Err(e) = save_window_state_sync(app) {
                eprintln!("Failed to save window state after creating new note: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to create window: {}", e);
        }
    }
}
