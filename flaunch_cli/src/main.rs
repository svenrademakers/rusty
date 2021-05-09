// extern crate clap;

// use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
// use flaunch_core::logging::*;
// use flaunch_core::script_engine::ScriptEngine;
// use flaunch_core::*;

// fn main() {
//     let modules = load_flaunch_core();
//     let script_engine = modules.0;
//     let settings = modules.1;

//     let scripts_path = settings.get_str(SettingKey::ScriptsDir).unwrap();
//     let matches = get_app_cli(scripts_path);
//     match matches.subcommand_name().unwrap() {
//         "list" => list_subcommand(script_engine.context),
//         "run" => run_subcommand(&matches, script_engine),
//         _ => {}
//     }
// }

// fn get_app_cli(scripts_path: &str) -> ArgMatches {
//     App::new(app_meta::APP_INFO.name)
//         .version(app_meta::VERSION)
//         .author(app_meta::APP_INFO.author)
//         .about(&*format!("run any script def in {}", scripts_path))
//         .arg(
//             Arg::with_name("config")
//                 .short("c")
//                 .long("config")
//                 .value_name("FILE")
//                 .help("Sets a custom config file")
//                 .takes_value(true),
//         )
//         .subcommand(
//             SubCommand::with_name("run")
//                 .about("run an arbitrary script")
//                 .arg(
//                     Arg::with_name("Script Name")
//                         .help("The configuration file to use")
//                         .index(1)
//                         .required(true),
//                 ),
//         )
//         .subcommand(SubCommand::with_name("list").about("list all available scripts"))
//         .setting(AppSettings::SubcommandRequired)
//         .get_matches()
// }

// fn list_subcommand(script_store: ScriptStore) {
//     for name in script_store.names {
//         println!("\t{}\t\t--\t{}", name.1, script_store.description[name.0]);
//     }
// }

// fn run_subcommand(matches: &ArgMatches, script_engine: ScriptEngine) {
//     let script_name = matches.value_of("Script Name").unwrap();

//     let arguments: Vec<Box<dyn std::any::Any>> = vec![Box::new("BAMI".to_string())];
//     if let Some(key) = script_engine.find(script_name) {
//         if script_engine.call(key, &arguments).unwrap() {
//             info!("Called {} successfully!", script_name);
//         }
//     } else {
//         println!("Script {} does not exist", script_name);
//         println!("Available scripts: ");
//         for name in script_engine.context.names {
//             println!("{:?}", name.1);
//         }
//     }
// }

fn main() {}
