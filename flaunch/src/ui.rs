#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[link(name = "flaunch_ui")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
use flaunch_core::app_meta;
use std::{ffi::CString, os::raw::c_char, thread};

fn to_c_char(string: &str) -> *const c_char {
    let ptr = CString::new(string).unwrap();
    ptr.into_raw() as *const c_char
}

pub fn start_ui() {
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

unsafe extern "C" fn Quit() {
    println!("quit application!");
}
