use crate::core::effect::Effect;
use crate::services::log_buffer::LogLevel;

use super::intent::ConsoleIntent;
use super::model::ConsoleModel;

pub fn apply(_model: &mut ConsoleModel, intent: &ConsoleIntent) -> Vec<Effect> {
    match intent {
        ConsoleIntent::Clear => {
            crate::services::log_buffer::clear_logs();
            vec![Effect::PushLog {
                level: LogLevel::Info,
                source: "console".to_string(),
                message: "console cleared".to_string(),
            }]
        }
    }
}
