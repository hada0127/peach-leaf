use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerData {
    pub id: String,
    pub file_path: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub background_color: String,
    pub text_color: String,
    pub mode: String,
    #[serde(default)]
    pub monitor_name: Option<String>,
    #[serde(default)]
    pub monitor_position: Option<(i32, i32)>,
    #[serde(default)]
    pub monitor_size: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub windows: Vec<StickerData>,
}
