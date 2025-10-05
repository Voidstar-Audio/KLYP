use std::path::PathBuf;

use crate::editor::{DurationPreset, RangePreset};
use platform_dirs::AppDirs;
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Preferences {
    pub duration_preset: DurationPreset,
    pub range_preset: RangePreset
}

fn preferences_file() -> PathBuf {
    let app_dirs = AppDirs::new(Some("Voidstar Audio"), false).unwrap();

    let mut config_path = app_dirs.config_dir;
    config_path.push("KLYP");
    config_path.push("config.json");
    config_path
}

fn preferences_dir() -> PathBuf {
    let app_dirs = AppDirs::new(Some("Voidstar Audio"), false).unwrap();

    let mut config_path = app_dirs.config_dir;
    config_path.push("KLYP");
    config_path
}

pub fn load_preferences() -> Preferences {
    std::fs::read_to_string(preferences_file())
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn store_preferences(preferences: &Preferences) {
    let _ = std::fs::create_dir_all(preferences_dir());
    if let Ok(prefs) = serde_json::to_string(preferences) {
        let _ = std::fs::write(preferences_file(), prefs).unwrap();
    }
}