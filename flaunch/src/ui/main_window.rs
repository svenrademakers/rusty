use std::rc::Rc;

use flaunch_core::{
    app_meta,
    script_engine::{Script, ScriptController},
};

use futures::channel::mpsc::Sender;
use gio::prelude::*;
use gtk::{prelude::*, WindowType};

use super::{app_models::ScriptEngineModelObserver, AppDataModels, Window, WindowBuilder};

pub struct MainWindow {
    script_listbox: gtk::ListBox,
    controller: Sender<ScriptController>,
}
impl Window for MainWindow {
    fn show(&self) {
        todo!()
    }

    fn hide(&self) {
        todo!()
    }
}

impl WindowBuilder for MainWindow {
    fn build(
        models: &mut AppDataModels,
        script_controller: Sender<ScriptController>,
    ) -> Rc<dyn Window> {
        let main_window = MainWindow {
            script_listbox: gtk::ListBox::new(),
            controller: script_controller,
        };

        let title = format!(
            "{} [{}] {}",
            app_meta::APP_NAME,
            app_meta::VERSION,
            app_meta::BUILD_DATE
        );
        let win = gtk::Window::new(WindowType::Toplevel);
        win.set_default_size(800, 600);

        let titlebar = gtk::HeaderBar::new();
        titlebar.set_show_close_button(true);
        titlebar.set_title(Some(title.as_str()));
        win.set_titlebar(&titlebar);

        win.add(&main_window.script_listbox);
        win.show_all();

        let window = Rc::new(main_window);
        models.subscribe_script_engine(&window);
        window
    }
}

impl ScriptEngineModelObserver for MainWindow {
    fn new_or_updated(&self, scripts: &Vec<Script>) {
        for script in scripts {
            println!("script incoming");
            let label = gtk::Button::new_with_label(&script.name);
            // let key = script.get_key().unwrap();
            // // label.connect_clicked(|_| {
            // //     self.controller
            // //         .try_send(ScriptController::Call(key, Vec::new()))
            // //         .unwrap();
            // // });
            self.script_listbox.add(&label);
        }
        self.script_listbox.show_all();
    }

    fn removed(&self, _key: u64) {}
}
