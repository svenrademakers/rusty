use glib::{subclass::types::ObjectSubclassExt, Cast};
use gtk::{gio, glib};

use crate::{
    application::{self},
    system_tray::win,
};

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;
}

impl MainWindow {
    pub fn new(app: &application::Application, resources: gio::Resource) -> Self {
        let window =
            glib::Object::new(&[("application", app)]).expect("Failed to create MainWindow");
        let w = imp::MainWindow::from_instance(&window);
        w.init(resources);
        window
    }
}

pub mod imp {
    use std::cell::RefCell;

    use crate::application::ScriptEngineCmd;
    use crate::application_controllers::{control, watch};
    use flaunch_core::app_meta;
    use flaunch_core::script_engine::{Script, ScriptChange};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{gio, glib};

    #[derive(Debug, Default)]
    pub struct MainWindow {
        script_listbox: gtk::ListBox,
        resource: RefCell<Option<gio::Resource>>,
    }

    impl MainWindow {
        pub fn init(&self, resource: gio::Resource) {
            *self.resource.borrow_mut() = Some(resource);
        }
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

            let wind = self.instance();
            watch(move |change| {
                match change {
                    ScriptChange::NewOrUpdated(vec) => on_new_script(&vec, &wind),
                    ScriptChange::Deleted(_x) => todo!(),
                }

                let priv_ = MainWindow::from_instance(&wind);
                priv_.script_listbox.show_all();
                true
            });

            let menu_refresh = gtk::MenuItem::with_label("Refresh");
            let menu_settings = gtk::MenuItem::with_label("Settings");
            menu_settings.connect_activate(move |_| {
                // let path = format!("file://{}", master_settings().to_string_lossy().to_string());
                // system_uri::open(path).unwrap();
            });

            let menu = gtk::MenuBar::new();
            menu.add(&menu_refresh);
            menu.add(&menu_settings);

            let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            v_box.add(&menu);
            v_box.add(&self.script_listbox);

            obj.add(&v_box);
            obj.show_all();
        }
    }

    fn on_new_script(vec: &Vec<Script>, wind: &super::MainWindow) {
        for ch in vec {
            let priv_ = MainWindow::from_instance(wind);
            priv_.script_listbox.add(&script_list_item(ch));
        }
    }

    fn script_list_item(script: &Script) -> gtk::Widget {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        // let image = gtk::Image::n
        hbox.add(&gtk::Button::with_label(">"));
        let btn = gtk::Button::with_label(script.name.as_str());
        let key = script.get_key().unwrap();
        btn.connect_clicked(move |_| {
            control(ScriptEngineCmd::Call(key));
        });
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        hbox.add(&btn);

        let mut args = String::new();
        for (name, typ, desc) in &script.arguments {
            args = format!("{} {}: {}", args, name, typ);
        }
        hbox.add(&gtk::Label::new(args.as_str().into()));
        hbox.set_tooltip_text(script.description.as_str().into());
        vbox.add(&hbox);
        return vbox.upcast::<gtk::Widget>();
    }

    impl WidgetImpl for MainWindow {}
    impl ContainerImpl for MainWindow {}
    impl BinImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
}
