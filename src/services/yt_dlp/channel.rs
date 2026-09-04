use std::sync::Arc;

use chrono::NaiveDate;
use serde_json::Value;

use crate::services::log_buffer::LogLevel;
use crate::services::traits::{
    ChannelOrder, ChannelPreview, ChannelProvider, ChannelVideo, LogSink, VideoKind,
};
use crate::services::yt_dlp::binary::{hidden_command, resolve_binary};

pub struct YtDlpChannelProvider {
    log: Arc<dyn LogSink>,
}

impl YtDlpChannelProvider {
    pub fn new(log: Arc<dyn LogSink>) -> Self {
        Self { log }
    }
}

#[async_trait::async_trait]
impl ChannelProvider for YtDlpChannelProvider {
    async fn preview(&self, order: &ChannelOrder) -> Result<ChannelPreview, String> {
        let url = order.url.trim();
        if url.is_empty() {
            return Err("channel_error_empty_url".to_string());
        }
        let binary = resolve_binary()?;
        let output = hidden_command(&binary)
            .arg("--flat-playlist")
            .arg("--dump-single-json")
            .arg("--no-warnings")
            .arg("--skip-download")
            .arg(url)
            .output()
            .await
            .map_err(|_| "channel_error_fetch".to_string())?;
        if !output.status.success() {
            self.log
                .push(LogLevel::Warn, "yt-dlp", "channel dump exited non-zero");
            return Err("channel_error_fetch".to_string());
        }
        let raw = String::from_utf8_lossy(&output.stdout);
        let value: Value =
            serde_json::from_str(&raw).map_err(|_| "channel_error_fetch".to_string())?;
        Ok(parse_channel_json(&value, order))
    }
}

/// Kind heuristic for flat-playlist entries. Shorts are detected by URL
/// (`/shorts/`); live content by `live_status` values observed in the
/// wild (`is_live`, `was_live`, `is_upcoming`, `upcoming`). This is a
/// documented approximation: flat playlists carry no stream-state
/// guarantee, so edge cases may classify as plain video.
pub fn classify_kind(url: &str, live_status: Option<&str>, title: &str) -> VideoKind {
    let _ = title;
    if url.contains("/shorts/") {
        return VideoKind::Short;
    }
    if matches!(
        live_status,
        Some("is_live" | "was_live" | "is_upcoming" | "upcoming")
    ) {
        return VideoKind::Live;
    }
    VideoKind::Video
}

/// Date filter for flat-playlist entries. A missing date means INCLUDE:
/// yt-dlp did not report it (private/deleted/upcoming entries often lack
/// `upload_date`), and dropping them would silently hide content.
pub fn within_range(
    date: Option<NaiveDate>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> bool {
    if let (Some(day), Some(start)) = (date, from) {
        if day < start {
            return false;
        }
    }
    if let (Some(day), Some(end)) = (date, to) {
        if day > end {
            return false;
        }
    }
    true
}

/// Pure mapping from `yt-dlp --flat-playlist --dump-single-json` to the
/// app DTO. Applies the order's kind/date filters; never fails, never
/// spawns a subprocess.
pub fn parse_channel_json(value: &Value, order: &ChannelOrder) -> ChannelPreview {
    let name =
        str_field(value, &["channel", "uploader", "uploader_id", "title"]).unwrap_or_default();
    let handle = str_field(value, &["uploader_id", "channel_id"]);
    let subs = value
        .get("channel_follower_count")
        .or_else(|| value.get("subscriber_count"))
        .or_else(|| value.get("follower_count"))
        .and_then(Value::as_u64)
        .map(format_count);
    let mut videos = Vec::new();
    if let Some(entries) = value.get("entries").and_then(Value::as_array) {
        for entry in entries {
            if let Some(video) = parse_entry(entry, order) {
                videos.push(video);
            }
        }
    }
    ChannelPreview {
        name,
        handle,
        subs,
        videos,
    }
}

fn parse_entry(entry: &Value, order: &ChannelOrder) -> Option<ChannelVideo> {
    if entry.is_null() {
        return None;
    }
    let id = str_field(entry, &["id"]).unwrap_or_default();
    let raw_url = str_field(entry, &["url"]).unwrap_or_default();
    let url = if raw_url.starts_with("http") {
        raw_url.clone()
    } else if !id.is_empty() {
        format!("https://www.youtube.com/watch?v={id}")
    } else {
        return None;
    };
    let title = str_field(entry, &["title"]).unwrap_or_default();
    let live_status = entry.get("live_status").and_then(Value::as_str);
    let kind = classify_kind(&url, live_status, &title);
    if !order.kinds.is_empty() && !order.kinds.contains(&kind) {
        return None;
    }
    let date = entry
        .get("upload_date")
        .and_then(Value::as_str)
        .and_then(parse_yyyymmdd);
    if !within_range(date, order.from, order.to) {
        return None;
    }
    let date_label = date.map(format_br_date).unwrap_or_else(|| "—".to_string());
    let duration_secs = entry
        .get("duration")
        .and_then(Value::as_f64)
        .map(|secs| secs.max(0.0) as u64)
        .unwrap_or(0);
    let id = if id.is_empty() { raw_url } else { id };
    Some(ChannelVideo {
        id,
        title,
        date_label,
        duration_secs,
        url,
        kind,
    })
}

fn str_field(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            if !text.trim().is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

fn parse_yyyymmdd(text: &str) -> Option<NaiveDate> {
    let text = text.trim();
    if text.len() != 8 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let year: i32 = text[0..4].parse().ok()?;
    let month: u32 = text[4..6].parse().ok()?;
    let day: u32 = text[6..8].parse().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
}

pub fn format_br_date(date: NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

pub fn format_count(total: u64) -> String {
    let amount = total as f64;
    if amount >= 1_000_000.0 {
        return format!("{}M", trim_float(format!("{:.2}", amount / 1_000_000.0)));
    }
    if amount >= 1_000.0 {
        return format!("{}K", trim_float(format!("{:.1}", amount / 1_000.0)));
    }
    total.to_string()
}

fn trim_float(text: String) -> String {
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn day(year: i32, month: u32, day: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(year, month, day)
    }

    fn open_order() -> ChannelOrder {
        ChannelOrder {
            url: "https://www.youtube.com/@chan/videos".to_string(),
            from: None,
            to: None,
            kinds: Vec::new(),
        }
    }

    #[test]
    fn kind_matrix() {
        let cases = [
            (
                "https://www.youtube.com/watch?v=abc",
                None,
                "Talk",
                VideoKind::Video,
            ),
            (
                "https://www.youtube.com/shorts/xyz",
                None,
                "Clip",
                VideoKind::Short,
            ),
            (
                "https://www.youtube.com/shorts/xyz",
                Some("is_live"),
                "Clip",
                VideoKind::Short,
            ),
            (
                "https://www.youtube.com/watch?v=abc",
                Some("is_live"),
                "Stream",
                VideoKind::Live,
            ),
            (
                "https://www.youtube.com/watch?v=abc",
                Some("was_live"),
                "Stream",
                VideoKind::Live,
            ),
            (
                "https://www.youtube.com/watch?v=abc",
                Some("is_upcoming"),
                "Premiere",
                VideoKind::Live,
            ),
            (
                "https://www.youtube.com/watch?v=abc",
                Some("upcoming"),
                "Premiere",
                VideoKind::Live,
            ),
            (
                "https://www.youtube.com/watch?v=abc",
                Some("not_live"),
                "Talk",
                VideoKind::Video,
            ),
            (
                "https://www.youtube.com/watch?v=abc",
                None,
                "Talk",
                VideoKind::Video,
            ),
        ];
        for (url, live, title, expected) in cases {
            assert_eq!(classify_kind(url, live, title), expected, "url {url}");
        }
    }

    #[test]
    fn range_filter_bounds_are_inclusive() {
        let from = day(2024, 1, 1);
        let to = day(2024, 1, 31);
        assert!(within_range(day(2024, 1, 1), from, to));
        assert!(within_range(day(2024, 1, 31), from, to));
        assert!(within_range(day(2024, 1, 15), from, to));
        assert!(!within_range(day(2023, 12, 31), from, to));
        assert!(!within_range(day(2024, 2, 1), from, to));
    }

    #[test]
    fn missing_date_is_included() {
        let from = day(2024, 1, 1);
        let to = day(2024, 1, 31);
        assert!(within_range(None, from, to));
        assert!(within_range(None, None, None));
    }

    #[test]
    fn counts_format_compact() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1_500), "1.5K");
        assert_eq!(format_count(2_000), "2K");
        assert_eq!(format_count(1_520_000), "1.52M");
        assert_eq!(format_count(3_000_000), "3M");
    }

    fn fixture() -> Value {
        serde_json::json!({
            "channel": "Demo Channel",
            "uploader_id": "demochannel",
            "channel_follower_count": 1_520_000,
            "entries": [
                {"id": "aaa", "title": "Old talk", "url": "aaa", "duration": 600.0, "upload_date": "20231201"},
                {"id": "bbb", "title": "New talk", "url": "bbb", "duration": 754.0, "upload_date": "20240115"},
                {"id": "ccc", "title": "Clip", "url": "https://www.youtube.com/shorts/ccc", "duration": 42.0, "upload_date": "20240120"},
                {"id": "ddd", "title": "Stream", "url": "ddd", "duration": 3600.0, "upload_date": "20240125", "live_status": "was_live"},
                {"id": "eee", "title": "Undated video", "url": "eee", "duration": 100.0},
                null,
                {"title": "Orphan entry"},
            ]
        })
    }

    #[test]
    fn parses_flat_playlist_fixture() {
        let preview = parse_channel_json(&fixture(), &open_order());
        assert_eq!(preview.name, "Demo Channel");
        assert_eq!(preview.handle.as_deref(), Some("demochannel"));
        assert_eq!(preview.subs.as_deref(), Some("1.52M"));
        assert_eq!(preview.videos.len(), 5);
        let clip = &preview.videos[2];
        assert_eq!(clip.kind, VideoKind::Short);
        assert_eq!(clip.url, "https://www.youtube.com/shorts/ccc");
        assert_eq!(clip.date_label, "20/01/2024");
        assert_eq!(preview.videos[1].url, "https://www.youtube.com/watch?v=bbb");
        assert_eq!(preview.videos[3].kind, VideoKind::Live);
        assert_eq!(preview.videos[4].date_label, "—");
    }

    #[test]
    fn applies_kind_and_date_filters() {
        let order = ChannelOrder {
            from: day(2024, 1, 1),
            to: day(2024, 1, 31),
            kinds: vec![VideoKind::Video],
            ..open_order()
        };
        let preview = parse_channel_json(&fixture(), &order);
        assert_eq!(preview.videos.len(), 2);
        assert!(preview.videos.iter().all(|v| v.kind == VideoKind::Video));
    }
}
