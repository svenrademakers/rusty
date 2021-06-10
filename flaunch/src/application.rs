use crate::main_window::{imp, MainWindow};
use crate::system_tray::run_system_tray_thread;
use flaunch_core::script_engine::ScriptChange;
use flaunch_core::{app_meta, run_logic_thread};
use gtk::gio;
use gtk::gio::ApplicationFlags;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use tokio::sync::{mpsc, watch};

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

        let app = app.downcast_ref::<Application>().unwrap();
        let priv_ = FlaunchApp::from_instance(app);
        let (tx_control, rx_control) = mpsc::channel(32);
        let (tx_script_m, rx_script_m) = watch::channel(ScriptChange::Deleted(0));
        let (tx_exit, rx_exit) = watch::channel(false);

        run_logic_thread(tx_script_m, rx_control, rx_exit);
        run_system_tray_thread();

        let window = MainWindow::new(&app);
        imp::MainWindow::from_instance(&window).subscribe_models(rx_script_m);
        window.show();
        priv_.window.set(window).unwrap();
    }
}

impl GtkApplicationImpl for FlaunchApp {}
