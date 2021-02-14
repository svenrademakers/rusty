mod app_meta;
use app_meta::*;

use hypter_core::logging::*;
use hypter_core::script_engine::{ScriptEngine, ScriptStore, Argument};
use hypter_core::settings::*;

extern crate clap;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

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

    let settings = Settings::new(&user_config_path, &setting_defaults());
    let scripts_path = settings.get_str(SettingKey::ScriptsDir).unwrap();
    info!("scripts_path: {}", scripts_path);

    let mut script_engine = ScriptEngine::new();
    script_engine.load(scripts_path).unwrap();

    let matches = get_app_cli(scripts_path);
    match matches.subcommand_name().unwrap() {
        "list" => list_subcommand(script_engine.context),
        "run" => run_subcommand(&matches, script_engine),
        _ => {}
    }
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

fn get_app_cli(scripts_path: &str) -> ArgMatches {
    App::new(APP_INFO.name)
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
        .subcommand(
            SubCommand::with_name("run")
                .about("run an arbitrary script")
                .arg(
                    Arg::with_name("Script Name")
                        .help("The configuration file to use")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("list all available scripts"))
        .setting(AppSettings::SubcommandRequired)
        .get_matches()
}

fn list_subcommand(script_store: ScriptStore) {
    for name in script_store.names {
        println!("\t{}\t\t--\t{}", name.1, script_store.description[name.0]);
    }
}

fn run_subcommand(matches: &ArgMatches, script_engine: ScriptEngine) {
    let script_name = matches.value_of("Script Name").unwrap();

    let arguments = vec![Argument::String("BAMI".to_string())];
    if let Some(key) = script_engine.find(script_name) {
        if script_engine.call(key, &arguments).unwrap() {
            info!("Called {} successfully!", script_name);
        }
    } else {
        println!("Script {} does not exist", script_name);
        println!("Available scripts: ");
        for name in script_engine.context.names {
            println!("{:?}", name.1);
        }
    }
}
