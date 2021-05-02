mod app_launcher;
mod system_tray;
use std::sync::mpsc::{self, Receiver, Sender};

use app_launcher::*;
use flaunch_core::{app_meta, load_flaunch_core, logging::*};
use flaunch_core::{load_logging, settings::*};
use system_tray::*;

fn setup_system_tray(launcher: &AppLauncher) -> StatusBar {
    let (tx, _rx): (Sender<String>, Receiver<String>) = mpsc::channel();
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
    system_tray
}

fn main() {
    load_logging();
    let launcher = AppLauncher::new();
    let mut system_tray = setup_system_tray(&launcher);
    system_tray.run(true);

    let (script_engine, settings) = load_flaunch_core();
}
