use crate::services::log_buffer::LogLevel;

#[derive(Debug, Clone)]
pub enum Effect {
    ReloadLocale(String),
    PushLog {
        level: LogLevel,
        source: String,
        message: String,
    },
}
