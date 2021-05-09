use self::mainview_controller::MainViewController;
use flaunch_core::{script_engine::ScriptEngineCmd, settings::Settings, SettingKey};
use mainview_controller::MAINVIEW_CONTROLLER;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};
mod mainview_controller;

pub fn init_controllers(
    script_cmd: Sender<ScriptEngineCmd>,
    settings: Rc<RefCell<Settings<SettingKey>>>,
) {
    MAINVIEW_CONTROLLER = RefCell::new(Some(MainViewController::new(script_cmd, settings)));
}
