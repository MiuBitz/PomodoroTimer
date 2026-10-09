use std::os::windows::process::CommandExt;
use std::{env, process::Command};
// Reuse the C# app's registry value to prevent duplicate startup entries.
pub fn set_startup(enabled: bool) -> Result<(), String> {
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    let command = format!("\"{}\" --autostart", exe.display()).replace('\'', "''");
    let action = if enabled {
        format!("if (-not (Test-Path $key)) {{ New-Item -Path $key -Force | Out-Null }}; Set-ItemProperty -Path $key -Name 'PomodoroTimer' -Value '{command}'")
    } else {
        "if (Get-ItemProperty -Path $key -Name 'PomodoroTimer' -ErrorAction SilentlyContinue) { Remove-ItemProperty -Path $key -Name 'PomodoroTimer' }".into()
    };
    let script = format!("$ErrorActionPreference = 'Stop'; $key = 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run'; {action}");
    let output = Command::new("powershell.exe")
        .creation_flags(0x08000000)
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .map_err(|e| format!("Startup update failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "Startup update failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
