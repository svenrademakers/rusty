pub mod app_meta;
pub mod logging;
pub mod script_engine;
pub mod settings;

use core::time;
use std::path::PathBuf;

use app_meta::*;
use futures::{executor::block_on, StreamExt};
use logging::*;
use script_engine::{ScriptChange, ScriptController, ScriptEngine};
use settings::*;
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::WatchStream;

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

pub fn run_logic_thread(
    sender: watch::Sender<ScriptChange>,
    mut controller: mpsc::Receiver<ScriptController>,
    exit_signal: watch::Receiver<bool>,
) {
    let thread = async move {
        let mut engine = ScriptEngine::new(sender);
        let settings = load_settings();

        // default load scripts-dir for now
        if let Some(script_path) = settings.get_str(SettingKey::ScriptsDir) {
            let path = PathBuf::from(script_path);
            engine.load(&path).await.unwrap();
        }

        let mut rx = WatchStream::new(exit_signal);
        tokio::select! {
            cmd = controller.recv() => controller_adapter(&mut engine, cmd.unwrap()).await,
            _ = rx.next() => return,
            else => std::thread::sleep(time::Duration::from_millis(100)),
        }
    };

    std::thread::spawn(|| block_on(thread));
}

async fn controller_adapter(engine: &mut ScriptEngine, cmd: ScriptController) {
    match cmd {
        ScriptController::Load(dir) => {
            engine.load(&dir).await.unwrap();
        }
        ScriptController::Call(key, args) => {
            engine.call(key, &args).unwrap();
        }
    }
}
