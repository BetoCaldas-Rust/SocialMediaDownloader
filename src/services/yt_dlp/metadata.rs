use std::sync::Arc;

use crate::services::traits::{LogSink, MetadataProvider, VideoMetadata};
use crate::services::yt_dlp::binary::{hidden_command, resolve_binary};

pub struct YtDlpMetadataProvider {
    log: Arc<dyn LogSink>,
}

impl YtDlpMetadataProvider {
    pub fn new(log: Arc<dyn LogSink>) -> Self {
        Self { log }
    }
}

#[async_trait::async_trait]
impl MetadataProvider for YtDlpMetadataProvider {
    async fn fetch_metadata(&self, url: &str) -> Result<VideoMetadata, String> {
        if url.trim().is_empty() {
            return Err("video_error_empty_url".to_string());
        }
        let binary = resolve_binary()?;
        let output = hidden_command(&binary)
            .arg("--dump-single-json")
            .arg("--no-playlist")
            .arg("--skip-download")
            .arg("--no-warnings")
            .arg(url.trim())
            .output()
            .await
            .map_err(|_| "video_error_fetch".to_string())?;
        if !output.status.success() {
            self.log.push(
                crate::services::log_buffer::LogLevel::Warn,
                "yt-dlp",
                "metadata dump exited non-zero",
            );
            return Err("video_error_fetch".to_string());
        }
        let raw = String::from_utf8_lossy(&output.stdout);
        let value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|_| "video_error_fetch".to_string())?;
        Ok(parse_metadata_json(&value))
    }
}

/// Pure mapping from `yt-dlp --dump-single-json` to the app DTO.
/// Missing fields degrade to empty/None; never fails.
pub fn parse_metadata_json(value: &serde_json::Value) -> VideoMetadata {
    let title = str_field(value, &["title"]).unwrap_or_default();
    let channel = str_field(value, &["channel", "uploader", "uploader_id"]).unwrap_or_default();
    let duration_secs = value
        .get("duration")
        .and_then(serde_json::Value::as_f64)
        .map(|secs| secs.max(0.0) as u64)
        .unwrap_or(0);
    let view_count = value.get("view_count").and_then(serde_json::Value::as_u64);
    let size_label = value
        .get("filesize_approx")
        .or_else(|| value.get("filesize"))
        .and_then(serde_json::Value::as_u64)
        .map(format_size);
    VideoMetadata {
        title,
        channel,
        duration_secs,
        view_count,
        size_label,
    }
}

fn str_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(text) = value.get(key).and_then(serde_json::Value::as_str) {
            if !text.trim().is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

fn format_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    let size = bytes as f64;
    if size >= GIB {
        format!("{:.2} GiB", size / GIB)
    } else if size >= MIB {
        format!("{:.1} MiB", size / MIB)
    } else if size >= KIB {
        format!("{:.0} KiB", size / KIB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_realistic_dump_json() {
        let raw = serde_json::json!({
            "title": "Some Video Title",
            "channel": "Some Channel",
            "duration": 754.0,
            "view_count": 123456,
            "filesize_approx": 89_400_000
        });
        let meta = parse_metadata_json(&raw);
        assert_eq!(meta.title, "Some Video Title");
        assert_eq!(meta.channel, "Some Channel");
        assert_eq!(meta.duration_secs, 754);
        assert_eq!(meta.view_count, Some(123456));
        assert_eq!(meta.size_label.as_deref(), Some("85.3 MiB"));
    }

    #[test]
    fn tolerates_missing_fields() {
        let meta = parse_metadata_json(&serde_json::json!({}));
        assert_eq!(meta.title, "");
        assert_eq!(meta.duration_secs, 0);
        assert_eq!(meta.view_count, None);
        assert_eq!(meta.size_label, None);
    }

    #[test]
    fn falls_back_to_uploader() {
        let meta = parse_metadata_json(&serde_json::json!({"uploader": "Solo Creator"}));
        assert_eq!(meta.channel, "Solo Creator");
    }
}
