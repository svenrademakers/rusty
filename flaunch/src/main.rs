mod app_launcher;
mod controllers;
mod system_tray;
mod utils;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

use std::{cell::RefCell, thread};
use std::{
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
};

use app_launcher::*;
use controllers::*;
use flaunch_core::{app_meta, load_settings, script_engine::ScriptEngine};
use flaunch_core::{load_logging, settings::*};
use flaunch_ui::run_gui_blocking;
use system_tray::*;

fn run_system_tray_thread() {
    thread::spawn(|| {
        let launcher = AppLauncher::new();
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

        system_tray.run(true);
    });
}

fn run_logic_thread() {
    thread::spawn(|| {
        let settings = Rc::new(RefCell::new(load_settings()));
        let script_engine = Rc::new(ScriptEngine::new());
        let mut controllers = Controllers::new(script_engine, settings);
        loop {
            controllers.poll();
        }
    });
}

fn main() {
    load_logging();
    run_logic_thread();
    run_system_tray_thread();
    run_gui_blocking();
}
