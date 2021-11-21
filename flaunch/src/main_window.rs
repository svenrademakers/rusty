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
    use crate::application::ScriptEngineCmd;
    use crate::application_controllers::{self, control, watch};
    use flaunch_core::app_meta;
    use flaunch_core::script_engine::{Script, ScriptChange};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Debug, Default)]
    pub struct MainWindow {
        script_listbox: gtk::ListBox,
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
        }
    }

    fn on_new_script(vec: &Vec<Script>, wind: &super::MainWindow) {
        for ch in vec {
            let priv_ = MainWindow::from_instance(wind);
            priv_.script_listbox.add(&script_list_item(ch));
        }
    }

    fn script_list_item(script: &Script) -> gtk::Widget {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

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
