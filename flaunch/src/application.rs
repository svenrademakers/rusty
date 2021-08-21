use std::sync::Arc;

use crate::application_controllers::register_sender_receiver;
use crate::main_window::MainWindow;
use crate::system_tray::run_system_tray_thread;
use flaunch_core::logging::{error, info};
use flaunch_core::script_engine::{ScriptChange, ScriptEngine};
use flaunch_core::settings::Settings;
use flaunch_core::{app_meta, load_settings};
use futures::executor::block_on;
use futures::pin_mut;
use gtk::gio;
use gtk::gio::ApplicationFlags;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use tokio::select;
use tokio::sync::mpsc::Receiver;

glib::wrapper! {
    pub struct Application(ObjectSubclass<FlaunchApp>)
        @extends gio::Application, gtk::Application;
}

impl Application {
    pub fn new() -> Self {
        let application_id = format!(
            "org.{}.{}",
            app_meta::APP_INFO.name,
            app_meta::APP_INFO.author
        );

        glib::Object::new(&[
            ("application-id", &application_id),
            ("flags", &ApplicationFlags::empty()),
        ])
        .unwrap()
    }
}

#[derive(Default)]
pub struct FlaunchApp {
    window: OnceCell<MainWindow>,
}

#[glib::object_subclass]
impl ObjectSubclass for FlaunchApp {
    const NAME: &'static str = app_meta::APP_NAME;
    type Type = Application;
    type ParentType = gtk::Application;
}

impl ObjectImpl for FlaunchApp {}

impl ApplicationImpl for FlaunchApp {
    fn activate(&self, app: &Self::Type) {
        let app = app.downcast_ref::<Application>().unwrap();
        let priv_ = FlaunchApp::from_instance(app);
        let window = priv_
            .window
            .get()
            .expect("Should always be initiliazed in gio_application_startup");
        window.show_all();
        window.present();
    }

    fn startup(&self, app: &Self::Type) {
        self.parent_startup(app);

        run_system_tray_thread();
        run_logic_thread();

        let self_ = self.instance();
        let window = MainWindow::new(&self_);
        let priv_ = FlaunchApp::from_instance(&self_);
        priv_.window.set(window).unwrap();
    }
}

impl GtkApplicationImpl for FlaunchApp {}

pub enum ScriptEngineCmd {
    Call(u64),
    Load,
}

impl std::fmt::Debug for ScriptEngineCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptEngineCmd::Call(x) => write!(f, "Call({})", x),
            ScriptEngineCmd::Load => write!(f, "Load"),
        }
    }
}

pub fn run_logic_thread() {
    let (s, r) = tokio::sync::watch::channel(ScriptChange::Deleted(0));
    let s_clone = s.clone();
    let r_clone = r.clone();

    register_sender_receiver(r);

    let (script_send, script_recv) = tokio::sync::mpsc::channel(64);
    register_sender_receiver(script_send.clone());

    std::thread::spawn(move || {
        futures::executor::block_on(async {
            let engine = ScriptEngine::new((s_clone, r_clone));
            let settings = load_settings();

            // Create a channel to receive the events.
            let (tx, rx) = channel();
            let mut watcher = raw_watcher(tx).unwrap();

            futures::select! {
                cmd = script_recv.recv() => {
                    match cmd {
                        ScriptEngineCmd::Call(key) => {
                            let _res = engine.call(key, &Vec::new()).unwrap();
                        }
                        ScriptEngineCmd::Load => {
                            if let Some(watching_dir) = load_cmd(&settings, &engine).await {
                                watcher.watch(&watching_dir, RecursiveMode::Recursive).unwrap();
                            }
                        }
                    }
                },
                file_change = rx.recv() => {
                    match file_change {
                        Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => engine.load_path(path),
                        Err(x) => error!(x.to_string()),
                    }
                },
            }

            info!("EXIT LOGIC");
        })
    });
}

async fn load_cmd(settings: &Settings<SettingKey>, engine: &ScriptEngine) -> Option<String> {
    if let Some(script_path) = settings.get_str(SettingKey::ScriptsDir) {
        let path = PathBuf::from(script_path);
        let fut = engine.load_path(&path);
        if cfg!(target_family = "unix") {
            if let Some(load_alias) = settings.get_bool(SettingKey::LoadAliases) {
                if load_alias {
                    let _ = futures::join!(engine.load_aliases(), fut);
                }
            }
        } else {
            fut.await.unwrap();
        }
        Some(script_path.to_string())
    }
    None
}
