// mod application_controller;
// mod script_engine_controller;

// use flaunch_core::{script_engine::ScriptEngine, settings::Settings, SettingKey};
// use flaunch_ui::SCRIPT_ENGINE_CMD;
// use script_engine_controller::ScriptEngineController;
// use std::{cell::RefCell, rc::Rc};

// pub trait Poll {
//     fn poll(&mut self);
// }

// pub struct Controllers {
//     controllers: Vec<Box<dyn Poll>>,
//     _settings: Rc<RefCell<Settings<SettingKey>>>,
// }

// impl Controllers {
//     pub fn new(
//         script_engine: Rc<ScriptEngine>,
//         settings: Rc<RefCell<Settings<SettingKey>>>,
//     ) -> Self {
//         let script_controller = ScriptEngineController::new(
//             script_engine,
//             settings.clone(),
//             SCRIPT_ENGINE_CMD.1.clone(),
//         );
//         Controllers {
//             controllers: vec![Box::new(script_controller)],
//             _settings: settings,
//         }
//     }

//     pub fn poll(&mut self) {
//         // let _w = self.settings.get_receiver().try_recv();
//         for controller in &mut self.controllers {
//             controller.poll();
//         }
//     }
// }
