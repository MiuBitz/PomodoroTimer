use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    #[serde(alias = "WorkMinutes")]
    pub work_minutes: u32,
    #[serde(alias = "BreakMinutes")]
    pub break_minutes: u32,
    #[serde(alias = "Cycles")]
    pub cycles: u32,
    #[serde(alias = "StartWithWindows")]
    pub start_with_windows: bool,
    #[serde(alias = "NotificationsEnabled")]
    pub notifications_enabled: bool,
    #[serde(alias = "SoundEnabled")]
    pub sound_enabled: bool,
    #[serde(alias = "MinimizeToTray")]
    pub minimize_to_tray: bool,
    pub theme_mode: String,
    pub color_preset: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            break_minutes: 5,
            cycles: 4,
            start_with_windows: false,
            notifications_enabled: true,
            sound_enabled: true,
            minimize_to_tray: true,
            theme_mode: "dark".into(),
            color_preset: "blue".into(),
        }
    }
}
impl Settings {
    fn path() -> Result<PathBuf, String> {
        std::env::var_os("APPDATA")
            .map(|p| PathBuf::from(p).join("PomodoroTimer").join("settings.json"))
            .ok_or("APPDATA unavailable".into())
    }
    pub fn load() -> Self {
        Self::path()
            .ok()
            .and_then(|p| fs::read(p).ok())
            .and_then(|s| serde_json::from_slice::<Self>(&s).ok())
            .filter(|s| s.validate().is_ok())
            .unwrap_or_default()
    }
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=180).contains(&self.work_minutes) {
            return Err("Work duration must be 1–180 minutes".into());
        }
        if !(1..=60).contains(&self.break_minutes) {
            return Err("Break duration must be 1–60 minutes".into());
        }
        if !(1..=99).contains(&self.cycles) {
            return Err("Cycles must be 1–99".into());
        }
        if !["dark", "light"].contains(&self.theme_mode.as_str())
            || !["violet", "blue", "sage", "amber"].contains(&self.color_preset.as_str())
        {
            return Err("Invalid appearance".into());
        }
        Ok(())
    }
    pub fn save(&self) -> Result<(), String> {
        self.validate()?;
        let path = Self::path()?;
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("tmp");
        fs::write(
            &tmp,
            serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::rename(tmp, path).map_err(|e| e.to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_settings() {
        let s: Settings = serde_json::from_str(r#"{"WorkMinutes":30,"Cycles":2}"#).unwrap();
        assert_eq!(s.work_minutes, 30);
        assert_eq!(s.cycles, 2);
        assert!(s.validate().is_ok());
    }
    #[test]
    fn reject_zero() {
        let s = Settings {
            work_minutes: 0,
            ..Settings::default()
        };
        assert!(s.validate().is_err());
    }
}
