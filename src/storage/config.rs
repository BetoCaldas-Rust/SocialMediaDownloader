use std::path::PathBuf;

use serde::{Deserialize, Serialize};

fn default_locale() -> String {
    "en-US".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(default = "default_locale")]
    pub locale: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            locale: default_locale(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|base| base.config_dir().join("SMD").join("config.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        serde_json::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return;
            }
        }
        let Ok(content) = serde_json::to_string_pretty(self) else {
            return;
        };
        if std::fs::write(&path, content).is_err() {
            tracing::warn!(target: "config", "unable to save config");
        }
    }
}
