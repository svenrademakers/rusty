mod app_launcher;
mod application;
mod main_window;
mod system_tray;
mod watch_pool;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

use application::Application;
use flaunch_core::load_logging;
use gtk::prelude::ApplicationExtManual;

fn main() {
    load_logging();
    gtk::init().expect("Failed to initialize gtk");
    let app = Application::new();
    app.run();
}
