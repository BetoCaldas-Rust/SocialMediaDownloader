use std::path::PathBuf;
use std::sync::RwLock;

use crate::services::traits::{HistoryEntry, HistoryStore};

pub const HISTORY_CAP: usize = 500;

pub struct JsonHistoryStore {
    path: PathBuf,
    entries: RwLock<Vec<HistoryEntry>>,
}

impl JsonHistoryStore {
    pub fn history_path() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|base| base.config_dir().join("SMD").join("history.json"))
    }

    pub fn new(path: PathBuf) -> Self {
        let entries = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<Vec<HistoryEntry>>(&content).ok())
            .unwrap_or_default();
        Self {
            path,
            entries: RwLock::new(entries),
        }
    }

    pub fn load() -> Self {
        Self::new(Self::history_path().unwrap_or_else(std::env::temp_dir))
    }

    fn save(&self) {
        let content = match self.entries.read() {
            Ok(guard) => match serde_json::to_string_pretty(&*guard) {
                Ok(content) => content,
                Err(_) => return,
            },
            Err(_) => return,
        };
        if let Some(parent) = self.path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return;
            }
        }
        if std::fs::write(&self.path, content).is_err() {
            tracing::warn!(target: "history", "unable to save history");
        }
    }
}

impl HistoryStore for JsonHistoryStore {
    fn entries(&self) -> Vec<HistoryEntry> {
        self.entries
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    fn record(&self, entry: HistoryEntry) {
        if let Ok(mut guard) = self.entries.write() {
            guard.push(entry);
            if guard.len() > HISTORY_CAP {
                let excess = guard.len() - HISTORY_CAP;
                guard.drain(..excess);
            }
        }
        self.save();
    }

    fn clear(&self) {
        if let Ok(mut guard) = self.entries.write() {
            guard.clear();
        }
        self.save();
    }

    fn len(&self) -> usize {
        self.entries.read().map(|guard| guard.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::traits::{
        new_history_id, now_ms, EntryKind, EntryStatus, TranscriptFormat, TranscriptOrder,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_path() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("smd-history-test-{}-{id}.json", std::process::id()))
    }

    fn sample_entry(name: &str) -> HistoryEntry {
        HistoryEntry {
            id: new_history_id(),
            kind: EntryKind::Transcript,
            name: name.to_string(),
            source: "youtube.com".to_string(),
            url: "https://www.youtube.com/watch?v=abc".to_string(),
            detail: Some("[pt] (auto)".to_string()),
            finished_at_ms: now_ms(),
            size_bytes: Some(128),
            status: EntryStatus::Completed,
            error: None,
            path: Some(PathBuf::from("/tmp/video [abc].srt")),
            transcript_order: Some(TranscriptOrder {
                url: "https://www.youtube.com/watch?v=abc".to_string(),
                lang: "pt".to_string(),
                format: TranscriptFormat::Srt,
                fallback: Some("en".to_string()),
                accept_auto: true,
                timestamps: true,
                output_dir: PathBuf::from("/tmp"),
            }),
        }
    }

    #[test]
    fn round_trip_preserves_entries_including_transcript_order() {
        let path = temp_path();
        let first = sample_entry("First");
        let mut second = sample_entry("Second");
        second.kind = EntryKind::Video;
        second.status = EntryStatus::Failed;
        second.error = Some("video_error_download".to_string());
        second.transcript_order = None;
        second.path = None;
        second.size_bytes = None;
        {
            let store = JsonHistoryStore::new(path.clone());
            store.record(first.clone());
            store.record(second.clone());
            assert_eq!(store.len(), 2);
        }
        let reloaded = JsonHistoryStore::new(path.clone());
        assert_eq!(reloaded.entries(), vec![first, second]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn cap_evicts_oldest_entries() {
        let path = temp_path();
        let store = JsonHistoryStore::new(path.clone());
        for index in 0..(HISTORY_CAP + 5) {
            store.record(sample_entry(&format!("Video {index}")));
        }
        let entries = store.entries();
        assert_eq!(entries.len(), HISTORY_CAP);
        assert_eq!(
            entries.first().map(|entry| entry.name.as_str()),
            Some("Video 5")
        );
        assert_eq!(
            entries.last().map(|entry| entry.name.as_str()),
            Some(&format!("Video {}", HISTORY_CAP + 4) as &str)
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn corrupt_file_loads_empty() {
        let path = temp_path();
        let _ = std::fs::write(&path, "not json{{{");
        let store = JsonHistoryStore::new(path.clone());
        assert!(store.entries().is_empty());
        let _ = std::fs::remove_file(&path);
    }
}
