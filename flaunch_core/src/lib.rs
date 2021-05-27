pub mod app_meta;
pub mod logging;
pub mod script_engine;
pub mod settings;

use std::cell::RefCell;

use app_meta::*;
use logging::*;
use script_engine::ScriptEngine;
use settings::*;
use tokio::join;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingKey {
    // path to the folder to watch. containing the actual scripts.
    ScriptsDir,
    FolderScan,
}

pub fn app_setting_defaults() -> Vec<KeyWithDefault<SettingKey>> {
    let mut dict: Vec<KeyWithDefault<SettingKey>> = Vec::new();

    // default scripts dir
    let mut scripts_dir = std::env::current_dir().unwrap_or_default();
    scripts_dir.push("test_scripts");
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

pub async fn load_core_components() -> (RefCell<Settings<SettingKey>>, ScriptEngine) {
    load_logging();

    let settings_task = async { RefCell::new(load_settings()) };
    let engine_task = async { ScriptEngine::new() };
    join!(settings_task, engine_task)
}

fn load_logging() {
    if let Err(e) = init_logging(LevelFilter::Debug) {
        println!("error initialising logger! {}", e);
    }

    info!("App:\t{}", APP_INFO.name);
    info!("Author: \t{}", APP_INFO.author);
    info!("Version:\t{} ({})", VERSION, BUILD_DATE);
    info!("--------------------------------------");
}

fn load_settings() -> Settings<SettingKey> {
    let mut settings = settings::Settings::new(&app_setting_defaults());
    settings.load();
    settings
}
