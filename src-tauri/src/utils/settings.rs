use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub close_to_tray: bool,
    pub auto_check_update: bool,
    #[serde(default = "default_quota_chart_type")]
    pub quota_chart_type: String,
}

fn default_quota_chart_type() -> String {
    "ring".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            close_to_tray: true,
            auto_check_update: true,
            quota_chart_type: "ring".to_string(),
        }
    }
}

pub fn get_settings_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("AntiVault");
    path.push("settings.json");
    path
}

pub fn load_settings() -> AppSettings {
    let path = get_settings_path();
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
            return settings;
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) {
    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let s = AppSettings::default();
        assert!(s.close_to_tray);
        assert!(s.auto_check_update);
        assert_eq!(s.quota_chart_type, "ring");
    }
}
