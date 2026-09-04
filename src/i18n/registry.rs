use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use super::loader;

pub struct I18nRegistry {
    locales_dir: PathBuf,
    available: Vec<String>,
    active_locale: String,
    active_map: HashMap<String, String>,
    fallback_map: HashMap<String, String>,
}

impl I18nRegistry {
    pub fn new(locales_dir: impl Into<PathBuf>) -> Self {
        let locales_dir = locales_dir.into();
        let available = loader::available_locales(&locales_dir);
        let fallback_map = loader::load_locale_map(&locales_dir, loader::FALLBACK_LOCALE);
        let active_map = fallback_map.clone();
        Self {
            locales_dir,
            available,
            active_locale: loader::FALLBACK_LOCALE.to_string(),
            active_map,
            fallback_map,
        }
    }

    pub fn available(&self) -> Vec<String> {
        self.available.clone()
    }

    pub fn current_locale(&self) -> String {
        self.active_locale.clone()
    }

    pub fn translate(&self, key: &str) -> String {
        if let Some(value) = self.active_map.get(key).cloned() {
            return value;
        }
        let locale = self.active_locale.clone();
        if let Some(value) = self.fallback_map.get(key).cloned() {
            tracing::warn!(target: "i18n", "missing key '{key}' for locale '{locale}'");
            return value;
        }
        tracing::warn!(target: "i18n", "missing key '{key}' without fallback");
        key.to_string()
    }

    pub fn set_locale(&mut self, locale: &str) -> bool {
        let known = self.available.iter().any(|item| item == locale);
        self.active_locale = locale.to_string();
        self.active_map = loader::load_locale_map(&self.locales_dir, locale);
        known
    }
}

static REGISTRY: OnceLock<RwLock<I18nRegistry>> = OnceLock::new();

pub fn init_i18n(locales_dir: PathBuf) {
    let _ = REGISTRY.set(RwLock::new(I18nRegistry::new(locales_dir)));
}

fn with_registry<F, T>(fallback: T, action: F) -> T
where
    F: FnOnce(&I18nRegistry) -> T,
{
    if let Some(registry) = REGISTRY.get() {
        if let Ok(guard) = registry.read() {
            return action(&guard);
        }
    }
    fallback
}

pub fn t(key: &str) -> String {
    with_registry(key.to_string(), |registry| registry.translate(key))
}

pub fn current_locale() -> String {
    with_registry(loader::FALLBACK_LOCALE.to_string(), |registry| {
        registry.current_locale()
    })
}

pub fn available_locales() -> Vec<String> {
    with_registry(Vec::new(), |registry| registry.available())
}

pub fn set_locale(locale: &str) -> bool {
    let Some(registry) = REGISTRY.get() else {
        return false;
    };
    let Ok(mut guard) = registry.write() else {
        return false;
    };
    let applied = guard.set_locale(locale);
    let active = guard.current_locale();
    drop(guard);
    persist_locale(&active);
    applied
}

fn persist_locale(locale: &str) {
    let mut config = crate::storage::config::AppConfig::load();
    config.locale = locale.to_string();
    config.save();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir_with_english() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("smd-i18n-test-{}-{id}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("en-US.toml"), "hello = \"Hello\"\n");
        dir
    }

    #[test]
    fn missing_key_returns_key_itself() {
        let dir = temp_dir_with_english();
        let registry = I18nRegistry::new(&dir);
        assert_eq!(registry.translate("hello"), "Hello");
        assert_eq!(registry.translate("nope_missing"), "nope_missing");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_locale_file_falls_back_to_english() {
        let dir = temp_dir_with_english();
        let mut registry = I18nRegistry::new(&dir);
        registry.set_locale("xx-YY");
        assert_eq!(registry.translate("hello"), "Hello");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
