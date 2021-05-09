mod app_launcher;
mod controllers;
mod system_tray;

mod ui;
use std::{cell::RefCell, path::PathBuf, sync::mpsc::channel, thread};
use std::{
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
};

use app_launcher::*;
use controllers::init_controllers;
use flaunch_core::{
    app_meta, load_settings,
    logging::error,
    script_engine::{ScriptEngine, ScriptEngineCmd},
};
use flaunch_core::{load_logging, settings::*};
use system_tray::*;
use ui::start_ui;

fn run_system_tray() {
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
}

fn script_engine_recv(cmd: ScriptEngineCmd, engine: &mut ScriptEngine) {
    match cmd {
        ScriptEngineCmd::Load { path } => {
            if path.exists() {
                engine.load(&path).unwrap();
            } else {
                error!("scripts path does not exits: {:?}", path);
            }
        }
        ScriptEngineCmd::Call { key } => {
            let _wat = engine.call(key, &Vec::new()).unwrap();
        }
    }
}

fn main() {
    load_logging();
    let system_tray = thread::spawn(|| run_system_tray());

    let settings = Rc::new(RefCell::new(load_settings()));
    let mut script_engine = ScriptEngine::new();

    let (rx_engine, tx_engine) = channel();
    init_controllers(rx_engine, settings);
    start_ui();

    loop {
        if let Ok(cmd) = tx_engine.try_recv() {
            script_engine_recv(cmd, &mut script_engine);
        }
    }

    //let scripts_path = settings.get_str(SettingKey::ScriptsDir).unwrap();
    //debug!("scripts path: {}", scripts_path);

    //ui::init(Rc::new(Some(script_engine)));
    //ui::mainloop();

    //system_tray.join().unwrap();
}
