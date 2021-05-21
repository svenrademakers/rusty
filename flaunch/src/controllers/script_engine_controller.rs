use std::{cell::RefCell, ffi::CString, path::PathBuf, rc::Rc};

use crossbeam::channel::Receiver;
use flaunch_core::{
    logging::error,
    logging::warn,
    script_engine::{Key, KeyData, ScriptEngine, ScriptKey},
    settings::Settings,
    SettingKey,
};
use flaunch_ui::ScriptEngineCmd;

use super::Poll;

pub struct ScriptEngineController {
    receiver: Receiver<ScriptEngineCmd>,
    settings: Rc<RefCell<Settings<SettingKey>>>,
    script_engine: Rc<ScriptEngine>,
}

impl ScriptEngineController {
    pub fn new(
        script_engine: Rc<ScriptEngine>,
        settings: Rc<RefCell<Settings<SettingKey>>>,
        receiver: Receiver<ScriptEngineCmd>,
    ) -> Self {
        ScriptEngineController {
            receiver: receiver,
            settings: settings,
            script_engine: script_engine,
        }
    }

    fn load(&mut self, cpath: CString) {
        let rstr = cpath.into_string().unwrap();
        let mut path: PathBuf = PathBuf::from(rstr);
        if !path.exists() {
            if let Ok(settings) = self.settings.try_borrow() {
                if let Some(script_path) = settings.get_str(SettingKey::ScriptsDir) {
                    path = PathBuf::from(script_path);
                }
            } else {
                warn!("could not borrow settings");
            }
        }

        if path.exists() {
            if let Some(engine) = Rc::get_mut(&mut self.script_engine) {
                engine.load(&path).unwrap();
            } else {
                error!("could not borrow script engine");
            }
        } else {
            error!("scripts path does not exits: {:?}", path);
        }
    }
}

impl Poll for ScriptEngineController {
    fn poll(&mut self) {
        if let Ok(cmd) = self.receiver.try_recv() {
            match cmd {
                ScriptEngineCmd::Load { path } => {
                    let cstring = CString::from(path);
                    self.load(cstring);
                }
                ScriptEngineCmd::Call { key } => {
                    let script_key = ScriptKey::from(KeyData::from_ffi(key));
                    if script_key.is_null() {
                        error!("could not parse script_key {} to actual key", key);
                        return;
                    }
                    let _wat = self.script_engine.call(script_key, &Vec::new()).unwrap();
                }
            }
        }
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
