use std::{
    any::Any,
    cell::RefCell,
    ffi::{CStr, CString},
    os::raw::c_char,
    path::PathBuf,
    rc::Rc,
};

use flaunch_core::{
    logging::error,
    logging::warn,
    script_engine::{CScript, Key, KeyData, ScriptChange, ScriptEngine, ScriptKey},
    settings::Settings,
    SettingKey,
};

use crate::controllers::*;
use crate::utils::sx::Global;
use crossbeam_channel::{unbounded, Receiver, Sender};

type SubscribeCb = unsafe extern "C" fn(key: u64, name: *const c_char);

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptEngineCmd {
    Load { path: CString },
    Call { key: u64 },
}

static mut SCRIPT_ENGINE_CMD: Option<Sender<ScriptEngineCmd>> = None;
static mut SCRIPT_ENGINE_MODEL: Option<Receiver<ScriptChange>> = None;

static OBSERVERS: Global<Vec<SubscribeCb>> = Global::new(Vec::new());

#[no_mangle]
pub unsafe extern "C" fn load_script(file: *const c_char) {
    if let Some(cmd) = &SCRIPT_ENGINE_CMD {
        cmd.send(ScriptEngineCmd::Load {
            path: CStr::from_ptr(file).to_owned(),
        })
        .unwrap();
    }
}

#[no_mangle]
pub unsafe extern "C" fn execute_script(script_key: u64) {
    if let Some(cmd) = &SCRIPT_ENGINE_CMD {
        let _arguments: Vec<Box<dyn Any>> = Vec::new();
        cmd.send(ScriptEngineCmd::Call { key: script_key }).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn observe_script_model(raw_func: SubscribeCb) {
    if let Some(mut vector) = OBSERVERS.try_borrow_mut() {
        vector.push(raw_func);
    }
}

#[no_mangle]
pub extern "C" fn unobserve_script_model(raw_func: SubscribeCb) {
    if let Some(mut vector) = OBSERVERS.try_borrow_mut() {
        if let Some(pos) = vector.iter().position(|x| *x == raw_func) {
            vector.remove(pos);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn dequeue_rust_data() {
    if let Some(recv) = &SCRIPT_ENGINE_MODEL {
        if let Some(vector) = OBSERVERS.try_borrow_mut() {
            if let Ok(data) = recv.try_recv() {
                for obs in &*vector {
                    match data {
                        ScriptChange::New(ref script) => obs(script.key, script.name.as_ptr()),
                        ScriptChange::Deleted(_) => (),
                    }
                }
            }
        }
    }
}

pub struct ScriptEngineController {
    receiver: Receiver<ScriptEngineCmd>,
    settings: Rc<RefCell<Settings<SettingKey>>>,
    script_engine: Rc<ScriptEngine>,
}

impl ScriptEngineController {
    pub fn new(
        script_engine: Rc<ScriptEngine>,
        settings: Rc<RefCell<Settings<SettingKey>>>,
    ) -> Self {
        let (tx, rx) = unbounded();
        unsafe {
            SCRIPT_ENGINE_CMD = Some(tx);
        }

        ScriptEngineController {
            receiver: rx,
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

impl Drop for ScriptEngineController {
    fn drop(&mut self) {
        unsafe {
            SCRIPT_ENGINE_CMD = None;
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
