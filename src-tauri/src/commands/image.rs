use std::fs;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};
use arboard::Clipboard;

/// Save pasted image to note's images folder
#[tauri::command]
pub async fn save_pasted_image(
    note_path: String,
    image_data: String,
    image_name: String
) -> Result<String, String> {
    println!("save_pasted_image called: note={}, name={}", note_path, image_name);

    // Get note directory
    let note_path = Path::new(&note_path);
    let note_dir = note_path.parent()
        .ok_or_else(|| "Could not get note directory".to_string())?;

    // Get note filename without extension
    let note_stem = note_path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid note filename".to_string())?;

    // Create images folder: ~/.peach-leaf/notes/{note_name}_images/
    let images_dir = note_dir.join(format!("{}_images", note_stem));
    fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    // Decode base64 image data
    let image_bytes = general_purpose::STANDARD.decode(&image_data)
        .map_err(|e| format!("Failed to decode image data: {}", e))?;

    // Save image file
    let image_path = images_dir.join(&image_name);
    fs::write(&image_path, image_bytes)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    // Return relative path for markdown: ./{note_name}_images/{image_name}
    let relative_path = format!("./{}_images/{}", note_stem, image_name);
    println!("Image saved successfully: {}", relative_path);

    Ok(relative_path)
}

/// Delete image file
#[tauri::command]
pub async fn delete_image(
    note_path: String,
    image_path: String
) -> Result<(), String> {
    println!("delete_image called: note={}, image={}", note_path, image_path);

    let note_path = Path::new(&note_path);
    let note_dir = note_path.parent()
        .ok_or_else(|| "Could not get note directory".to_string())?;

    // Convert relative path to absolute
    let full_image_path = if image_path.starts_with("./") {
        note_dir.join(&image_path[2..])
    } else if image_path.starts_with("../") {
        note_dir.join(&image_path[3..])
    } else {
        note_dir.join(&image_path)
    };

    // Delete image file if it exists
    if full_image_path.exists() {
        fs::remove_file(&full_image_path)
            .map_err(|e| format!("Failed to delete image: {}", e))?;
        println!("Image deleted: {:?}", full_image_path);

        // Check if parent directory is empty and delete it
        if let Some(parent) = full_image_path.parent() {
            cleanup_empty_image_folder(parent)?;
        }
    }

    Ok(())
}

/// Cleanup empty image folder
fn cleanup_empty_image_folder(folder_path: &Path) -> Result<(), String> {
    // Check if folder ends with "_images"
    if let Some(folder_name) = folder_path.file_name().and_then(|s| s.to_str()) {
        if folder_name.ends_with("_images") {
            // Check if folder is empty
            let entries = fs::read_dir(folder_path)
                .map_err(|e| format!("Failed to read directory: {}", e))?;

            if entries.count() == 0 {
                fs::remove_dir(folder_path)
                    .map_err(|e| format!("Failed to remove empty directory: {}", e))?;
                println!("Removed empty image folder: {:?}", folder_path);
            }
        }
    }

    Ok(())
}

/// Cleanup all empty image folders for a note
#[tauri::command]
pub async fn cleanup_note_images(note_path: String) -> Result<(), String> {
    println!("cleanup_note_images called: note={}", note_path);

    let note_path = Path::new(&note_path);
    let note_dir = note_path.parent()
        .ok_or_else(|| "Could not get note directory".to_string())?;

    let note_stem = note_path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid note filename".to_string())?;

    let images_dir = note_dir.join(format!("{}_images", note_stem));

    if images_dir.exists() {
        cleanup_empty_image_folder(&images_dir)?;
    }

    Ok(())
}

/// Read image file as base64 data URL for preview
#[tauri::command]
pub async fn read_image_as_data_url(image_path: String) -> Result<String, String> {
    println!("read_image_as_data_url called: {}", image_path);

    let path = Path::new(&image_path);

    if !path.exists() {
        return Err(format!("Image file not found: {}", image_path));
    }

    // Read image file
    let image_bytes = fs::read(path)
        .map_err(|e| format!("Failed to read image file: {}", e))?;

    // Convert to base64
    let base64_data = general_purpose::STANDARD.encode(&image_bytes);

    // Determine MIME type from extension
    let mime_type = match path.extension().and_then(|s| s.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/png", // default
    };

    // Return as data URL
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);
    println!("Image converted to data URL successfully ({}KB)", image_bytes.len() / 1024);
    Ok(data_url)
}

/// Read image from clipboard using native clipboard access
#[tauri::command]
pub async fn read_clipboard_image(note_path: String) -> Result<Option<String>, String> {
    println!("read_clipboard_image called for note: {}", note_path);

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    // Try to get image from clipboard
    match clipboard.get_image() {
        Ok(img) => {
            println!("Image found in clipboard: {}x{}", img.width, img.height);

            // Convert RGBA bytes to PNG format
            let mut png_data = Vec::new();
            {
                use std::io::Cursor;
                let mut encoder = png::Encoder::new(
                    Cursor::new(&mut png_data),
                    img.width as u32,
                    img.height as u32,
                );
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);

                let mut writer = encoder.write_header()
                    .map_err(|e| format!("Failed to create PNG encoder: {}", e))?;

                writer.write_image_data(&img.bytes)
                    .map_err(|e| format!("Failed to encode PNG: {}", e))?;
            }

            // Convert to base64
            let base64_data = general_purpose::STANDARD.encode(&png_data);

            // Generate filename
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let filename = format!("image-{}.png", timestamp);

            // Save the image
            let relative_path = save_pasted_image(note_path, base64_data, filename).await?;

            println!("Image saved via native clipboard: {}", relative_path);
            Ok(Some(relative_path))
        }
        Err(_) => {
            println!("No image in clipboard");
            Ok(None)
        }
    }
}
