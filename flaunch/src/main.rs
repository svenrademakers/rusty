mod app_launcher;
mod system_tray;
mod ui;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

use std::path::PathBuf;

use app_launcher::*;
use flaunch_core::script_engine::{ScriptChange, ScriptController, ScriptEngine};
use flaunch_core::{app_meta, load_settings, SettingKey};
use flaunch_core::{load_logging, settings::*};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::executor::block_on;
use futures::{select, StreamExt};
use system_tray::*;
use ui::Application;

fn run_system_tray_thread() {
    std::thread::spawn(|| {
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

pub fn run_logic_thread(sender: Sender<ScriptChange>, mut controller: Receiver<ScriptController>) {
    std::thread::spawn(move || {
        block_on(async {
            let mut engine = ScriptEngine::new(sender);
            let settings = load_settings();

            // default load scripts-dir for now
            if let Some(script_path) = settings.get_str(SettingKey::ScriptsDir) {
                let path = PathBuf::from(script_path);
                engine.load(&path).await.unwrap();
            }

            loop {
                select! {
                    cmd = controller.select_next_some() => controller_adapter(&mut engine, cmd).await,
                    complete=>break,
                }
            }
        });
    });
}

pub fn run_application(recv: Receiver<ScriptChange>, controller: Sender<ScriptController>) {
    // let application_id = format!(
    //     "org.{}.{}",
    //     app_meta::APP_INFO.name,
    //     app_meta::APP_INFO.author
    // );

    // let uiapp = gtk::Application::new(
    //     Some(application_id.as_str()),
    //     gio::ApplicationFlags::FLAGS_NONE,
    // )
    // .unwrap();

    // uiapp.connect_activate(|app| {
    //     let (x, r) = channel(4);
    //     let (x2, r2) = channel(4);

    //     let mut app_container = Application::new(r, x2, app);
    //     app_container.run();
    // });

    // uiapp.run(&env::args().collect::<Vec<_>>());
    let mut app = Application::new(recv, controller);
    app.run();
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

fn main() {
    load_logging();
    run_system_tray_thread();

    let (engine_tx, engine_rx) = channel(32);
    let (script_controller_tx, script_controller_rx) = channel(32);
    run_logic_thread(engine_tx, script_controller_rx);
    run_application(engine_rx, script_controller_tx);
}
