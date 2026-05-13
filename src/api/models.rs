use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
    pub custom_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadResponse {
    pub id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadStatus {
    pub id: String,
    pub status: String,
    pub progress: f32,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub filename: Option<String>,
    pub platform: Option<String>,
    pub error: Option<String>,
    pub relative_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_path: String,
    pub platform_paths: HashMap<String, String>,
    pub temporary: bool,
    pub cookies_from_browser: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigUpdateRequest {
    pub default_path: Option<String>,
    pub platform_paths: Option<HashMap<String, String>>,
    pub temporary: bool,
    pub cookies_from_browser: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformInfo {
    pub name: String,
    pub url: String,
    pub supported: bool,
    pub folder_name: String,
}
