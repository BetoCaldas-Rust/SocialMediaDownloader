use std::collections::VecDeque;
use std::fmt;
use std::sync::{OnceLock, RwLock};

use tracing::field::Field;
use tracing::{Event, Subscriber};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::reload::{Handle, Layer as ReloadLayer};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
}

const MAX_ENTRIES: usize = 5000;

static BUFFER: OnceLock<RwLock<VecDeque<LogEntry>>> = OnceLock::new();

fn buffer() -> &'static RwLock<VecDeque<LogEntry>> {
    BUFFER.get_or_init(|| RwLock::new(VecDeque::new()))
}

fn stamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

pub fn push_log(level: LogLevel, source: &str, message: &str) {
    let entry = LogEntry {
        timestamp: stamp(),
        level,
        source: source.to_string(),
        message: message.to_string(),
    };
    if let Ok(mut guard) = buffer().write() {
        guard.push_back(entry);
        while guard.len() > MAX_ENTRIES {
            guard.pop_front();
        }
    }
}

pub fn log_entries() -> Vec<LogEntry> {
    buffer()
        .read()
        .map(|guard| guard.iter().cloned().collect())
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn log_len() -> usize {
    buffer().read().map(|guard| guard.len()).unwrap_or(0)
}

pub fn clear_logs() {
    if let Ok(mut guard) = buffer().write() {
        guard.clear();
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}

pub struct LogBufferLayer;

impl<S> Layer<S> for LogBufferLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let level = match *event.metadata().level() {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            _ => LogLevel::Debug,
        };
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        push_log(level, event.metadata().target(), &visitor.message);
    }
}

pub type LevelHandle = Handle<LevelFilter, Registry>;

pub fn init_tracing(initial: LevelFilter) -> LevelHandle {
    let (filter, handle) = ReloadLayer::new(initial);
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(LogBufferLayer)
        .try_init();
    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_caps_at_5000_entries() {
        clear_logs();
        for index in 0..5100 {
            push_log(LogLevel::Info, "cap-test", &format!("bulk-{index}"));
        }
        assert_eq!(log_len(), MAX_ENTRIES);
    }
}
