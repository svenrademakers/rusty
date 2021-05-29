pub mod main_window;

use std::rc::Rc;

pub mod app_models;
pub use app_models::*;
use flaunch_core::script_engine::{ScriptChange, ScriptController};
use futures::channel::mpsc::{Receiver, Sender};
pub use main_window::*;

pub trait Window {
    fn show(&self);
    fn hide(&self);
}

pub trait WindowBuilder {
    fn build(
        models: &mut AppDataModels,
        script_controller: Sender<ScriptController>,
    ) -> Rc<dyn Window>;
}

pub struct Application {
    models: AppDataModels,
    script_controller: Sender<ScriptController>,
    windows: Vec<Rc<dyn Window>>,
}

impl Application {
    pub fn new(
        script_model: Receiver<ScriptChange>,
        script_controller: Sender<ScriptController>,
    ) -> Self {
        Application {
            models: AppDataModels::new(script_model),
            script_controller: script_controller,
            windows: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
            return;
        }

        self.register_window::<MainWindow>();

        loop {
            unsafe {
                if gtk_sys::gtk_main_iteration_do(glib_sys::GFALSE) == glib_sys::GFALSE {
                    break;
                }
                self.models.poll();
            }
        }
    }

    pub fn register_window<T: WindowBuilder + Window>(&mut self) {
        let window = T::build(&mut self.models, self.script_controller.clone());
        self.windows.push(window);
    }
}
