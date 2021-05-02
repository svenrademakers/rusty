#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[link(name = "flaunch_ui")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
use root::ui::*;
use std::ffi::CString;
use std::os::raw::*;

pub fn to_c_char(string: &str) -> *const c_char {
    let ptr = CString::new(string).unwrap();
    ptr.into_raw() as *const c_char
}
pub type ScriptEngineRc = Rc<ScriptEngine>;
static mut SCRIPT_ENGINE: Option<ScriptEngineRc> = None;

pub fn init(engine: ScriptEngineArc) {
    SCRIPT_ENGINE = Some(engine.clone());

    unsafe {
        root::ui::init(
            to_c_char(app_meta::VERSION),
            to_c_char(app_meta::BUILD_DATE),
        );

        for script in &engine.context.scripts {
            add_script(
                script.0.data().as_ffi(),
                to_c_char(&engine.context.names[script.0]),
                Some(FLaunchApplication::execute_script),
            );
        }
    }
}

pub fn mainloop(&mut self) {
    unsafe {
        root::ui::mainloop();
    }
}

unsafe extern "C" fn execute_script(script_key: u64) {
    if let Some(engine) = &SCRIPT_ENGINE {
        let key = ScriptKey::from(KeyData::from_ffi(script_key));
        if key.is_null() {
            error!("could not parse script_key {} to actual key", script_key);
            return;
        }

        let arguments: Vec<Box<dyn Any>> = Vec::new();
        if let Err(e) = engine.call(key, &arguments) {
            error!(
                "calling {} {} ({}-{:?}) failed. {}",
                engine.context.scripts[key].to_string().to_lowercase(),
                engine.context.names[key],
                script_key,
                key,
                e
            );
        } else {
            info!("successfully called {}", engine.context.names[key])
        }
    }
}

unsafe extern "C" fn Quit() {
    println!("quit application!");
}
