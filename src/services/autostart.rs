use std::os::windows::process::CommandExt;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "SMD";

fn reg() -> std::process::Command {
    let mut cmd = std::process::Command::new("reg");
    cmd.creation_flags(0x08000000);
    cmd
}

fn app_path() -> Option<String> {
    std::env::current_exe().ok()?.to_str().map(str::to_string)
}

pub fn is_enabled() -> Option<bool> {
    let output = reg()
        .arg("query")
        .arg(RUN_KEY)
        .arg("/v")
        .arg(VALUE_NAME)
        .output()
        .ok()?;
    if output.status.success() {
        Some(true)
    } else {
        Some(false)
    }
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    if enabled {
        let Some(path) = app_path() else {
            return Err("autostart registry update failed".to_string());
        };
        let status = reg()
            .arg("add")
            .arg(RUN_KEY)
            .arg("/v")
            .arg(VALUE_NAME)
            .arg("/t")
            .arg("REG_SZ")
            .arg("/d")
            .arg(format!("\"{path}\""))
            .arg("/f")
            .status()
            .map_err(|_| "autostart registry update failed".to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("autostart registry update failed".to_string())
        }
    } else {
        let status = reg()
            .arg("delete")
            .arg(RUN_KEY)
            .arg("/v")
            .arg(VALUE_NAME)
            .arg("/f")
            .status()
            .map_err(|_| "autostart registry update failed".to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("autostart registry update failed".to_string())
        }
    }
}
