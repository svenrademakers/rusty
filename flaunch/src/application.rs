use std::cell::RefCell;
use std::rc::Rc;

use crate::main_window::MainWindow;
use crate::system_tray::run_system_tray_thread;
use flaunch_core::script_engine::{ScriptChange, ScriptEngine};
use flaunch_core::{app_meta, load_core_components};
use gtk::gio;
use gtk::gio::ApplicationFlags;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use tokio::sync::watch;

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
    pub script_engine: Rc<RefCell<ScriptEngine>>,
    pub script_model: OnceCell<watch::Receiver<ScriptChange>>,
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

        let app = app.downcast_ref::<Application>().unwrap();
        let priv_ = FlaunchApp::from_instance(app);

        let window = MainWindow::new(&app);
        priv_.window.set(window).unwrap();

        let context = glib::MainContext::default();
        let self_ = self.instance();
        context.spawn_local(async move {
            let engine = load_core_components().await;
            let receiver = engine.observe();
            let app = FlaunchApp::from_instance(&self_);
            app.script_engine.replace(engine);

            app.window
                .get()
                .unwrap()
                .init(receiver, app.script_engine.clone());
        });
    }
}

impl GtkApplicationImpl for FlaunchApp {}

// static PROPERTIES: [subclass::Property; 2] = [
//     subclass::Property("script_model", |name| {
//         glib::ParamSpec::object(
//             name,
//             "script_model",
//             "no idea what this is",
//             glib::types::Type::OBJECT(size_of<watch::Receiver<ScriptChange>>()),
//             glib::ParamFlags::READWRITE,
//         )
//     })
// ];
