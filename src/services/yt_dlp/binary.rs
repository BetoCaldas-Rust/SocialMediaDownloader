use std::path::PathBuf;

/// Locale key used as the error value when no sidecar is found.
/// Callers store it and the view translates it via `t()`.
pub const SIDECAR_MISSING_KEY: &str = "video_error_sidecar_missing";

fn exe_name() -> &'static str {
    if cfg!(windows) {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    }
}

/// Spawn helper shared by metadata and download calls.
/// The window must never flash a terminal on Windows, hence
/// `CREATE_NO_WINDOW` (0x08000000).
pub fn hidden_command(program: &PathBuf) -> tokio::process::Command {
    let mut command = tokio::process::Command::new(program);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    command
}

/// Lookup order (first hit wins):
/// 1. `resources/bin/` next to the running exe (dev: `<repo>/resources/bin/`)
/// 2. `bin/` next to the running exe (installed layout from cargo-packager)
/// 3. exe name next to the running exe
/// 4. `%APPDATA%\SMD\updates\` (F8 self-update drop zone)
/// 5. `yt-dlp` on PATH
pub fn resolve_binary() -> Result<PathBuf, String> {
    for candidate in candidates() {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(SIDECAR_MISSING_KEY.to_string())
}

fn candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("resources").join("bin").join(exe_name()));
            out.push(dir.join("bin").join(exe_name()));
            out.push(dir.join(exe_name()));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let dev = cwd.join("resources").join("bin").join(exe_name());
        if !out.contains(&dev) {
            out.push(dev);
        }
    }
    if let Some(base) = directories::BaseDirs::new() {
        out.push(base.data_dir().join("SMD").join("updates").join(exe_name()));
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(exe_name());
            if !out.contains(&candidate) {
                out.push(candidate);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_binary_reports_localized_key() {
        let result = resolve_binary();
        if let Err(key) = result {
            assert_eq!(key, SIDECAR_MISSING_KEY);
        }
    }

    #[test]
    fn installed_bin_dir_is_candidate() {
        let wanted = std::path::Path::new("bin").join(exe_name());
        assert!(candidates().iter().any(|path| path.ends_with(&wanted)));
    }
}
