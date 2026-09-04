use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const FALLBACK_LOCALE: &str = "en-US";

pub fn resolve_locales_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let next_to_exe = dir.join("locales");
            if next_to_exe.is_dir() {
                return next_to_exe;
            }
        }
    }
    PathBuf::from("locales")
}

pub fn available_locales(dir: &Path) -> Vec<String> {
    let mut locales = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return locales;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_toml = path.extension().and_then(|ext| ext.to_str()) == Some("toml");
        if is_toml {
            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                locales.push(stem.to_string());
            }
        }
    }
    locales.sort();
    locales
}

pub fn load_locale_map(dir: &Path, locale: &str) -> HashMap<String, String> {
    let path = dir.join(format!("{locale}.toml"));
    let Ok(content) = std::fs::read_to_string(&path) else {
        return HashMap::new();
    };
    let Ok(table) = toml::from_str::<toml::Table>(&content) else {
        return HashMap::new();
    };
    let mut map = HashMap::new();
    for (key, value) in table {
        if let Some(text) = value.as_str() {
            map.insert(key, text.to_string());
        }
    }
    map
}
