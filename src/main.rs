#[macro_use]
extern crate lazy_static;
extern crate notify;

use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;
mod app_meta;
mod logging;
mod script_engine;
use script_engine::ScriptManager;

mod settings;

use app_meta::*;
use logging::*;

fn start_script_watcher(watch_dir: String) {
    let dir = std::path::Path::new(&watch_dir);
    if !dir.exists() {
        std::fs::create_dir(dir).unwrap();
    }

    let (tx, rx) = channel();
    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(3)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(watch_dir, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn main() {
    init_logging(LevelFilter::Debug).unwrap();
    info!("App:\t{}", APP_INFO.name);
    info!("Author: \t{}", APP_INFO.author);
    info!("Version:\t{} ({})", VERSION, BUILD_DATE);
    info!("--------------------------------------");

    // let app_settings: settings::Settings = settings::Settings::new();

    // if let Some(path) = app_settings.scripts_path() {
    //     info!("Start watcher: {}", &path);
    //     let path_clone = path.clone();
    //     std::thread::spawn(move || start_script_watcher(path_clone));
    // }

    // loop {}

    let manager = ScriptManager::new();
}
