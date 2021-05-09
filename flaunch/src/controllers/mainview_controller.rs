use std::{
    any::Any,
    borrow::Borrow,
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};

use flaunch_core::{
    logging::error,
    script_engine::{Key, KeyData, ScriptEngineCmd, ScriptKey},
    settings::Settings,
    SettingKey,
};

pub static MAINVIEW_CONTROLLER: RefCell<Option<MainViewController>> = RefCell::new(None);

pub struct MainViewController {
    sender: Sender<ScriptEngineCmd>,
    settings: Rc<RefCell<Settings<SettingKey>>>,
}

impl MainViewController {
    pub fn new(
        script_cmds: Sender<ScriptEngineCmd>,
        settings: Rc<RefCell<Settings<SettingKey>>>,
    ) -> Self {
        MainViewController {
            sender: script_cmds,
            settings: settings,
        }
    }

    pub fn load_all(&mut self) {
        if let Ok(settings) = self.settings.try_borrow() {
            if let Some(path) = settings.get_str(SettingKey::ScriptsDir) {
                let cmd = ScriptEngineCmd::Load {
                    path: PathBuf::from(path),
                };
                self.sender.send(cmd);
            }
        }
    }

    pub fn call(&self, script_key: u64, args: &[Box<dyn Any>]) {
        let key = ScriptKey::from(KeyData::from_ffi(script_key));
        if key.is_null() {
            error!("could not parse script_key {} to actual key", script_key);
            return;
        }
        self.sender.send(ScriptEngineCmd::Call { key: key });
    }
}

unsafe impl Sync for MainViewController {}

unsafe extern "C" fn execute_script(script_key: u64) {
    if let Some(main_view) = MAINVIEW_CONTROLLER.borrow() {
        let arguments: Vec<Box<dyn Any>> = Vec::new();
        main_view.call(script_key, &arguments);
    }
}

// if let Err(e) = main_view.call(key, &arguments) {
//     error!(
//         "calling {} {} ({}-{:?}) failed. {}",
//         engine.context.scripts[key].to_string().to_lowercase(),
//         engine.context.names[key],
//         script_key,
//         key,
//         e
//     );
// } else {
//     info!("successfully called {}", engine.context.names[key])
// }
