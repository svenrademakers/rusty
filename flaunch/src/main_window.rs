use gtk::glib;

use crate::application::{self};

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

impl MainWindow {
    pub fn new(app: &application::Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create MainWindow")
    }
}

pub mod imp {
    use crate::watch_pool;
    use flaunch_core::app_meta;
    use flaunch_core::script_engine::{ScriptChange, ScriptController};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::unsync::OnceCell;
    use tokio::sync::{mpsc, watch};

    #[derive(Debug, Default)]
    pub struct MainWindow {
        script_listbox: gtk::ListBox,
        script_controller: OnceCell<mpsc::Sender<ScriptController>>,
        script_model: OnceCell<watch::Receiver<ScriptChange>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "MainWindow";
        type Type = super::MainWindow;
        type ParentType = gtk::ApplicationWindow;
    }

    impl ObjectImpl for MainWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let title = format!(
                "{} [{}] {}",
                app_meta::APP_NAME,
                app_meta::VERSION,
                app_meta::BUILD_DATE
            );
            let titlebar = gtk::HeaderBar::new();
            titlebar.set_show_close_button(true);
            titlebar.set_title(Some(title.as_str()));
            obj.set_titlebar(Some(&titlebar));

            obj.set_default_size(800, 600);
            obj.add(&self.script_listbox);
            obj.show_all();
        }
    }

    impl MainWindow {
        pub fn subscribe_models(&self, recv: watch::Receiver<ScriptChange>) {
            let wind = self.instance();
            watch_pool::watch(recv, move |change| {
                let priv_ = MainWindow::from_instance(&wind);
                match change {
                    ScriptChange::NewOrUpdated(vec) => {
                        for ch in vec {
                            let btn = gtk::Button::with_label(ch.name.as_str());
                            priv_.script_listbox.add(&btn);
                        }

                        priv_.script_listbox.show_all();
                    }
                    ScriptChange::Deleted(x) => (),
                }

                true
            });
        }
    }

    impl WidgetImpl for MainWindow {}
    impl ContainerImpl for MainWindow {}
    impl BinImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
}
