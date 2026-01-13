use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ResizeRequest {
    pub s3_url: String,
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_object_mode")]
    pub object_mode: ObjectMode,
}

fn default_object_mode() -> ObjectMode {
    ObjectMode::Cover
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ObjectMode {
    Cover,
    Contain,
    Fill,
    ScaleDown,
}

#[derive(Debug, Serialize)]
pub struct ResizeResponse {
    pub original_url: String,
    pub resized_url: String,
    pub width: u32,
    pub height: u32,
    pub object_mode: ObjectMode,
}
