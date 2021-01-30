#[macro_use]
extern crate lazy_static;
extern crate notify;
mod app_meta;
mod filewatcher;
mod logging;
mod script_engine;
mod settings;

use json::JsonValue;
use settings::KeyWithDefault;
use settings::Settings;

use std::hash::Hash;
use std::path::PathBuf;

use app_meta::*;
use logging::*;

lazy_static! {
    static ref APP_CONFIG: PathBuf =
        app_dirs::get_app_root(app_dirs::AppDataType::UserConfig, &app_meta::APP_INFO).unwrap();
    static ref CONF_FILENAME: String = {
        let mut config = APP_CONFIG.clone();
        config.push("config.json");
        config.to_string_lossy().to_string()
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingKey {
    ScriptsDir,
    FolderScan,
}

fn setting_defaults() -> Vec<KeyWithDefault<SettingKey>> {
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

fn main() {
    init_logging(LevelFilter::Debug).unwrap();
    info!("App:\t{}", APP_INFO.name);
    info!("Author: \t{}", APP_INFO.author);
    info!("Version:\t{} ({})", VERSION, BUILD_DATE);
    info!("--------------------------------------");

    let app_settings: Settings<SettingKey> = Settings::new(&CONF_FILENAME, &setting_defaults());

    if let Some(x) = app_settings.get_str(SettingKey::ScriptsDir) {
        info!("Start watcher: {}", x);
        let path_clone = x.to_string();
        std::thread::spawn(move || filewatcher::start_script_watcher(path_clone));
        // for entry in read_dir(path).unwrap() {
        //     let entry = entry?;
        //     if entry.is_file() {}
        // }
    }

    loop {}

    //let manager = ScriptManager::new();
}
