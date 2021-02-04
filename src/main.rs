mod Settings;
mod app_meta;
mod logging;
mod script_engine;

use app_meta::*;
use logging::*;
use script_engine::*;
use Settings::*;

extern crate clap;
use clap::{App, Arg};

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

    let user_config_path =
        app_dirs::get_app_root(app_dirs::AppDataType::UserConfig, &app_meta::APP_INFO)
            .map(|path| {
                let mut config = path;
                config.push("config.json");
                config.to_string_lossy().to_string()
            })
            .unwrap();

    let settings = Settings::Settings::new(&user_config_path, &setting_defaults());
    let scripts_path = settings.get_str(SettingKey::ScriptsDir).unwrap();
    info!("scripts_path: {}", scripts_path);
    let matches = App::new(APP_INFO.name)
        .version(VERSION)
        .author(APP_INFO.author)
        .about(&*format!("run any script def in {}", scripts_path))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Script Name")
                .help("Name of the script to run")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    let script_name = matches.value_of("Script Name").unwrap();
    let mut script_engine = ScriptEngine::new();
    script_engine.load(scripts_path);

    // if let Some(key) = script_engine.find(script_name) {
    //     if script_engine.call(key, &Vec::new()) {
    //         info!("Called {} successfully!", script_name);
    //     }
    // }
}
