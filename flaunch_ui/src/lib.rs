#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod system_tray;

#[link(name = "flaunch_ui")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
pub use root::ui::*;
pub use std::ffi::CString;
pub use std::os::raw::*;

pub fn to_c_char(string: &str) -> *const c_char {
    let ptr = CString::new(string).unwrap();
    ptr.into_raw() as *const c_char
}

