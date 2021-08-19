pub mod app_meta;
pub mod logging;
pub mod script_engine;
pub mod settings;

use std::path::PathBuf;

use app_meta::*;
use logging::*;
use script_engine::ScriptEngine;
use settings::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingKey {
    // path to the folder to watch. containing the actual scripts.
    ScriptsDir,
    FolderScan,
    LoadAliases,
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

    dict.push((SettingKey::LoadAliases, "aliases", JsonValue::Boolean(true)));

    dict
}

pub fn load_logging() {
    if let Err(e) = init_logging(LevelFilter::Debug) {
        println!("error initialising logger! {}", e);
    }

    info!("App:\t{}", APP_INFO.name);
    info!("Author: \t{}", APP_INFO.author);
    info!("Version:\t{} ({})", VERSION, BUILD_DATE);
    info!("--------------------------------------");
}

pub fn load_settings() -> Settings<SettingKey> {
    let mut settings = settings::Settings::new(&app_setting_defaults());
    settings.load();
    settings
}

pub async fn load_core_components() -> ScriptEngine {
    let engine = ScriptEngine::default();
    let settings = load_settings();

    // default load scripts-dir for now
    if let Some(script_path) = settings.get_str(SettingKey::ScriptsDir) {
        let path = PathBuf::from(script_path);
        let fut = engine.load_path(&path);

        if let Some(load_alias) = settings.get_bool(SettingKey::LoadAliases) {
            if load_alias {
                #[cfg(target_family = "unix")]
                let _ = futures::join!(engine.load_aliases(), fut);
            }
        } else {
            fut.await.unwrap();
        }
    }

    engine
}
