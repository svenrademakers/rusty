#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use std::{
    any::Any,
    ffi::{CStr, CString},
    os::raw::c_char,
    thread,
};

use crossbeam::channel::{unbounded, Receiver, Sender};
use flaunch_core::{app_meta, logging::error, script_engine::ScriptChange};
use lazy_static::*;

#[link(name = "flaunch_ui")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

//type SubscribeCb = unsafe extern "C" fn(key: u64, name: *const c_char);
lazy_static! {
    pub static ref SCRIPT_ENGINE_CMD: (Sender<ScriptEngineCmd>, Receiver<ScriptEngineCmd>) =
        unbounded();
    pub static ref SCRIPT_ENGINE_MODEL: (Sender<ScriptChange>, Receiver<ScriptChange>) =
        unbounded();
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptEngineCmd {
    Load { path: CString },
    Call { key: u64 },
}

fn to_c_char(string: &str) -> *const c_char {
    let ptr = CString::new(string).unwrap();
    ptr.into_raw() as *const c_char
}

pub fn run_gui_blocking() {
    unsafe {
        root::ui::init(
            to_c_char(app_meta::VERSION),
            to_c_char(app_meta::BUILD_DATE),
        );

        thread::spawn(|| {
            root::ui::mainloop();
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn load_script(file: *const c_char) {
    SCRIPT_ENGINE_CMD
        .0
        .send(ScriptEngineCmd::Load {
            path: CStr::from_ptr(file).to_owned(),
        })
        .unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn execute_script(script_key: u64) {
    let _arguments: Vec<Box<dyn Any>> = Vec::new();
    SCRIPT_ENGINE_CMD
        .0
        .send(ScriptEngineCmd::Call { key: script_key })
        .unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn poll_rust_data() {
    if let Ok(data) = SCRIPT_ENGINE_MODEL.1.try_recv() {
        match data {
            ScriptChange::New(ref script) => {
                root::ui::script_change_new(script.key, script.name.as_ptr())
            }
            ScriptChange::Deleted(key) => root::ui::script_change_delete(key),
        }
    }
}

#[no_mangle]
unsafe extern "C" fn Quit() {
    println!("quit application!");
}

#[no_mangle]
pub extern "C" fn log_error(message: *const ::std::os::raw::c_char) {
    unsafe {
        match std::ffi::CString::from_raw(message as *mut i8).into_string() {
            Ok(text) => error!("{}", text),
            Err(e) => println!("{}", e.to_string()),
        }
    }
}
