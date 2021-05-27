mod app_launcher;
mod system_tray;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

use std::path::PathBuf;

use app_launcher::*;
use flaunch_core::logging::info;
use flaunch_core::{app_meta, SettingKey};
use flaunch_core::{load_core_components, settings::*};
use system_tray::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::spawn_blocking;

fn run_system_tray_thread() {
    spawn_blocking(|| {
        let launcher = AppLauncher::new();
        let (tx, _rx): (Sender<String>, Receiver<String>) = channel(64);
        let mut system_tray = launcher.build_system_tray(tx);
        let cb: NSCallback = Box::new(move |_sender, _tx| {
            let path = format!("file://{}", master_settings().to_string_lossy().to_string());
            system_uri::open(path).unwrap();
        });
        let _ = system_tray.add_item(None, "Open Config", cb, false);
        system_tray.add_separator();
        system_tray.add_label(&format!(
            "{} {}[{}]",
            app_meta::APP_NAME,
            app_meta::BUILD_DATE,
            app_meta::VERSION
        ));
        system_tray.add_quit("Quit");

        system_tray.run(true);
    });
}

#[tokio::main]
async fn main() {
    let (settings, engine) = load_core_components().await;

    load_all_scripts(settings, engine).await;

    run_system_tray_thread();
}

async fn load_all_scripts(
    settings: std::cell::RefCell<Settings<SettingKey>>,
    mut engine: flaunch_core::script_engine::ScriptEngine,
) {
    if let Ok(set) = settings.try_borrow() {
        if let Some(script_path) = set.get_str(SettingKey::ScriptsDir) {
            let path = PathBuf::from(script_path);
            engine.load(&path).await.unwrap();
            if engine.find("Sven_for_life").is_some() {
                info!("what a time to be alive");
            }
        }
    }
}
