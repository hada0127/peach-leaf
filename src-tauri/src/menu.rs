use tauri::menu::{Menu, MenuBuilder, SubmenuBuilder, MenuItemBuilder, CheckMenuItemBuilder, CheckMenuItem};
use tauri::{Manager, Emitter};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Store for font menu items
pub static FONT_MENU_ITEMS: once_cell::sync::Lazy<Arc<Mutex<Option<FontMenuItems>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

pub struct FontMenuItems {
    pub small: CheckMenuItem<tauri::Wry>,
    pub medium: CheckMenuItem<tauri::Wry>,
    pub large: CheckMenuItem<tauri::Wry>,
    pub xlarge: CheckMenuItem<tauri::Wry>,
}

pub fn create_menu(app: &tauri::App) -> Result<Menu<tauri::Wry>, tauri::Error> {
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
        .separator()
        .item(&MenuItemBuilder::new("Print...").id("print").accelerator("CmdOrCtrl+P").build(app)?)
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

    // Font Menu with check items (default is Medium - 14px)
    let font_small = CheckMenuItemBuilder::with_id("font_small", "Small (12px)")
        .checked(false)
        .build(app)?;
    let font_medium = CheckMenuItemBuilder::with_id("font_medium", "Medium (14px)")
        .checked(true)
        .build(app)?;
    let font_large = CheckMenuItemBuilder::with_id("font_large", "Large (16px)")
        .checked(false)
        .build(app)?;
    let font_xlarge = CheckMenuItemBuilder::with_id("font_xlarge", "Extra Large (18px)")
        .checked(false)
        .build(app)?;

    // Store font menu items for later access
    {
        let mut items = FONT_MENU_ITEMS.lock().unwrap();
        *items = Some(FontMenuItems {
            small: font_small.clone(),
            medium: font_medium.clone(),
            large: font_large.clone(),
            xlarge: font_xlarge.clone(),
        });
    }

    // Style Menu - combines color and font options (color first)
    let style_menu = SubmenuBuilder::new(app, "Style")
        .item(&MenuItemBuilder::new("Choose Color...").id("open_color_picker").build(app)?)
        .separator()
        .item(&font_small)
        .item(&font_medium)
        .item(&font_large)
        .item(&font_xlarge)
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
        .item(&style_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()?;

    #[cfg(not(target_os = "macos"))]
    let menu = MenuBuilder::new(app)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&style_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()?;

    Ok(menu)
}

/// Update font menu checks based on font size
pub fn update_font_menu_checks(font_size: u32) {
    if let Some(ref items) = *FONT_MENU_ITEMS.lock().unwrap() {
        let _ = items.small.set_checked(font_size == 12);
        let _ = items.medium.set_checked(font_size == 14);
        let _ = items.large.set_checked(font_size == 16);
        let _ = items.xlarge.set_checked(font_size == 18);
    }
}

pub fn setup_menu_handler(app: &tauri::AppHandle) {
    use crate::window_manager::create_new_note_backend;

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

        // Handle print: emit to focused window only
        if menu_id == "print" {
            println!("Handling print in backend");
            if let Some(focused_window) = app.webview_windows().values().find(|w| {
                w.is_focused().unwrap_or(false)
            }) {
                let window_label = focused_window.label().to_string();
                println!("Emitting print event to focused window: {}", window_label);
                let _ = focused_window.emit(&format!("print_{}", window_label), ());
            }
            return;
        }

        // Handle font menu items: update check state
        if menu_id.starts_with("font_") {
            let font_size = match menu_id {
                "font_small" => 12,
                "font_medium" => 14,
                "font_large" => 16,
                "font_xlarge" => 18,
                _ => 14,
            };

            // Update menu check state
            update_font_menu_checks(font_size);
        }

        // Emit event to the focused window (for other menu items) - use window-specific event
        if let Some(focused_window) = app.webview_windows().values().find(|w| {
            w.is_focused().unwrap_or(false)
        }) {
            let window_label = focused_window.label();
            let event_name = format!("menu_{}", window_label);
            println!("Emitting to focused window: {} with event: {}", window_label, event_name);
            let _ = focused_window.emit(&event_name, menu_id);
        } else {
            // Fallback to main window if no window is focused
            if let Some(window) = app.get_webview_window("main") {
                println!("No focused window, emitting to main");
                let _ = window.emit("menu_main", menu_id);
            }
        }
    });
}
