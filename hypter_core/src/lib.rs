pub mod logging;
pub mod script_engine;
pub mod settings;

use settings::{JsonValue, KeyWithDefault};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingKey {
    ScriptsDir,
    FolderScan,
}

pub fn app_setting_defaults() -> Vec<KeyWithDefault<SettingKey>> {
    let mut dict: Vec<KeyWithDefault<SettingKey>> = Vec::new();

    // default scripts dir
    let mut scripts_dir = std::env::current_dir().unwrap_or_default();
    scripts_dir.push("scripts");
    dict.push((
        SettingKey::ScriptsDir,
        "scripts_dir",
        JsonValue::String(scripts_dir.to_string_lossy().to_string()),
    ));

    // default folder scan
    dict.push((
        SettingKey::FolderScan,
        "folder_scan",
        JsonValue::Boolean(true),
    ));

    dict
}
