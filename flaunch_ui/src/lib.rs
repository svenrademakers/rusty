#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[link(name = "flaunch_ui")]
include!(concat!(env!("OUT_DIR"), "/flaunch_ui_bindings.rs"));
use std::ffi::CString;
use std::os::raw::c_char;

pub fn to_c_char(string : &str) -> *const c_char 
{
    let ptr = CString::new(string).unwrap();
    ptr.into_raw() as *const c_char
}