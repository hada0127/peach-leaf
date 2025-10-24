use tauri::{Manager, RunEvent};

// Module declarations
mod models;
mod state;
mod commands;
mod menu;
mod window_manager;

// Re-export for external use if needed
pub use models::{StickerData, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::file::read_file,
            commands::file::write_file,
            commands::file::select_file,
            commands::file::delete_note_file,
            commands::window::create_sticker_window,
            commands::color::open_color_picker,
            commands::color::close_color_picker,
            commands::color::apply_color,
            commands::window::save_window_state,
            commands::window::get_saved_state,
            commands::window::get_window_data,
            commands::window::update_window_metadata,
            commands::window::on_window_focus
        ])
        .setup(|app| {
            // Create menu
            let menu = menu::create_menu(app)?;
            app.set_menu(menu)?;

            // Restore saved windows
            let app_handle = app.app_handle();
            match state::load_app_state() {
                Ok(state) => {
                    // Clean up orphaned note files before restoring windows
                    if let Err(e) = state::cleanup_orphaned_notes(&state) {
                        eprintln!("Failed to cleanup orphaned notes: {}", e);
                    }

                    if state.windows.is_empty() {
                        println!("No saved windows, creating default window");
                        if let Err(e) = window_manager::create_main_window(&app_handle) {
                            eprintln!("Failed to create main window: {}", e);
                        }
                    } else {
                        println!("Restoring {} saved windows", state.windows.len());
                        for window_data in state.windows {
                            window_manager::restore_window(&app_handle, window_data);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load saved state: {}", e);
                    if let Err(e) = window_manager::create_main_window(&app_handle) {
                        eprintln!("Failed to create main window: {}", e);
                    }
                }
            }

            // Setup menu event handler
            menu::setup_menu_handler(&app_handle);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                // Prevent app from exiting when all windows are closed
                RunEvent::ExitRequested { api, .. } => {
                    api.prevent_exit();
                }
                // Save state when a window is destroyed
                RunEvent::WindowEvent { label, event: window_event, .. } => {
                    if let tauri::WindowEvent::Destroyed = window_event {
                        println!("Window {} was destroyed, saving state...", label);
                        let app = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = commands::window::save_window_state(app).await {
                                eprintln!("Failed to save window state after window close: {}", e);
                            }
                        });
                    }
                }
                _ => {}
            }
        });
}
