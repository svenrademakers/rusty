use std::{cell::RefCell, rc::Rc};

use flaunch_core::script_engine::{ScriptChange, ScriptEngine};
use glib::subclass::types::ObjectSubclassExt;
use gtk::glib;
use tokio::sync::watch;

use crate::application::{self};

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

impl MainWindow {
    pub fn new(app: &application::Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create MainWindow")
    }

    pub fn init(
        &self,
        script_model: watch::Receiver<ScriptChange>,
        script_engine: Rc<RefCell<ScriptEngine>>,
    ) {
        let priv_ = imp::MainWindow::from_instance(self);
        priv_.init(script_model, script_engine);
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::watch_pool;
    use flaunch_core::app_meta;
    use flaunch_core::script_engine::{Script, ScriptChange, ScriptEngine};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::OnceCell;
    use tokio::sync::watch;

    #[derive(Debug, Default)]
    pub struct MainWindow {
        script_listbox: gtk::ListBox,
        script_engine: OnceCell<Rc<RefCell<ScriptEngine>>>,
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
        pub fn init(
            &self,
            script_model: watch::Receiver<ScriptChange>,
            script_engine: Rc<RefCell<ScriptEngine>>,
        ) {
            self.script_engine.set(script_engine).unwrap();
            self.watch_script_model_changes(script_model);
        }

        fn watch_script_model_changes(&self, script_model: watch::Receiver<ScriptChange>) {
            let wind = self.instance();

            watch_pool::watch(script_model, move |change| {
                match change {
                    ScriptChange::NewOrUpdated(vec) => on_new_script(vec, wind.clone()),
                    ScriptChange::Deleted(_x) => todo!(),
                }

                let priv_ = MainWindow::from_instance(&wind);
                priv_.script_listbox.show_all();
                true
            });
        }
    }

    fn on_new_script(vec: &Vec<Script>, wind: super::MainWindow) {
        for ch in vec {
            let btn = gtk::Button::with_label(ch.name.as_str());
            let key = ch.get_key().unwrap();
            let w = wind.clone();
            btn.connect_clicked(move |_| {
                let priv_ = MainWindow::from_instance(&w);
                priv_
                    .script_engine
                    .get()
                    .unwrap()
                    .borrow()
                    .call(key, &Vec::new())
                    .unwrap();
            });

            let priv_ = MainWindow::from_instance(&wind);
            priv_.script_listbox.add(&btn);
        }
    }

    impl WidgetImpl for MainWindow {}
    impl ContainerImpl for MainWindow {}
    impl BinImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
}
