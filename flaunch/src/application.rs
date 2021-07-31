use crate::application_controllers::register_sender_receiver;
use crate::main_window::MainWindow;
use crate::system_tray::run_system_tray_thread;
use flaunch_core::logging::info;
use flaunch_core::script_engine::ScriptEngine;
use flaunch_core::{app_meta, load_core_components};
use futures::executor::block_on;
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

        let self_ = self.instance();
        let engine = block_on(load_core_components());
        register_sender_receiver(engine.observe());

        let (script_send, script_recv) = tokio::sync::mpsc::channel(64);
        register_sender_receiver(script_send);
        run_logic_thread(engine, script_recv);

        let window = MainWindow::new(&self_);
        let priv_ = FlaunchApp::from_instance(&self_);
        priv_.window.set(window).unwrap();
    }
}

impl GtkApplicationImpl for FlaunchApp {}

pub enum ScriptEngineCmd {
    Call(u64),
}

impl std::fmt::Debug for ScriptEngineCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptEngineCmd::Call(x) => write!(f, "Call({})", x),
        }
    }
}

pub fn run_logic_thread(engine: ScriptEngine, mut script_cmd: Receiver<ScriptEngineCmd>) {
    std::thread::spawn(move || {
        futures::executor::block_on(async {
            loop {
                let cmd = script_cmd.recv().await.unwrap();
                process_cmd(&engine, cmd);
            }
        })
    });
}

fn process_cmd(engine: &ScriptEngine, cmd: ScriptEngineCmd) {
    match cmd {
        ScriptEngineCmd::Call(key) => {
            let _res = engine.call(key, &Vec::new()).unwrap();
        }
    }
}