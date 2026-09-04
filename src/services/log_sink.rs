use super::log_buffer::{push_log, LogLevel};
use super::traits::LogSink;

pub struct BufferLogSink;

impl LogSink for BufferLogSink {
    fn push(&self, level: LogLevel, source: &str, message: &str) {
        push_log(level, source, message);
    }
}
