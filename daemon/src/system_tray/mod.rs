#![allow(unused_variables)]

use flaunch_core::{app_meta, settings::master_settings};
use futures::channel::mpsc::{channel, Receiver, Sender};

use crate::app_launcher::{AppLauncher, AppLauncherT};

#[cfg(target_os = "macos")]
pub mod osx;

#[cfg(target_os = "windows")]
pub mod win;

#[cfg(target_os = "linux")]
pub type Object = u64;
#[cfg(target_os = "windows")]
pub type Object = u32;
#[cfg(target_os = "macos")]
pub type Object = osx::Object;

#[cfg(target_os = "linux")]
pub type StatusBar = DummyStatusBar;
#[cfg(target_os = "macos")]
pub type StatusBar = osx::OSXStatusBar;
#[cfg(target_os = "windows")]
pub type StatusBar = win::WindowsStatusBar;

pub trait TStatusBar {
    type S: TStatusBar;
    fn new(tx: Sender<String>, title: &str, icon_name: &str) -> Self::S;
    fn can_redraw(&mut self) -> bool;
    fn clear_items(&mut self);
    fn add_separator(&mut self);
    fn add_label(&mut self, label: &str);
    fn add_submenu(&mut self, label: &str, callback: NSCallback) -> *mut Object;
    fn add_item(
        &mut self,
        menu: Option<*mut Object>,
        item: &str,
        callback: NSCallback,
        selected: bool,
    ) -> *mut Object;
    fn add_quit(&mut self, label: &str);
    fn update_item(&mut self, item: *mut Object, label: &str);
    fn sel_item(&mut self, sender: u64);
    fn unsel_item(&mut self, sender: u64);
    fn set_tooltip(&mut self, text: &str);
    fn register_url_handler(&mut self);
    fn run(&mut self, block: bool);
}

pub type NSCallback = Box<dyn Fn(u64, &Sender<String>)>;

pub struct DummyStatusBar {}
impl TStatusBar for DummyStatusBar {
    type S = DummyStatusBar;
    fn new(tx: Sender<String>, title: &str, icon_name: &str) -> Self::S {
        DummyStatusBar {}
    }
    fn can_redraw(&mut self) -> bool {
        true
    }
    fn clear_items(&mut self) {}
    fn add_separator(&mut self) {}
    fn add_submenu(&mut self, _: &str, _: NSCallback) -> *mut Object {
        0 as *mut Object
    }
    fn add_label(&mut self, _: &str) {}
    fn add_item(&mut self, _: Option<*mut Object>, _: &str, _: NSCallback, _: bool) -> *mut Object {
        0 as *mut Object
    }
    fn add_quit(&mut self, _: &str) {}
    fn update_item(&mut self, _: *mut Object, _: &str) {}
    fn sel_item(&mut self, _: u64) {}
    fn unsel_item(&mut self, _: u64) {}
    fn set_tooltip(&mut self, _: &str) {}
    fn register_url_handler(&mut self) {}
    fn run(&mut self, _: bool) {}
}

pub fn run_system_tray_thread() {
    std::thread::spawn(|| {
        let launcher = AppLauncher::new();
        let (tx, _rx): (Sender<String>, Receiver<String>) = channel(64);
        let mut system_tray = launcher.build_system_tray(tx);
        let cb: NSCallback = Box::new(move |_sender, _tx| {
            let path = format!("file://{}", master_settings().to_string_lossy().to_string());
            system_uri::open(path).unwrap();
        });
        let _ = system_tray.add_item(None, "Open Config", cb, false);
        system_tray.add_separator();
        system_tray.add_label(&format!(
            "{} {}[{}]",
            app_meta::APP_NAME,
            app_meta::BUILD_DATE,
            app_meta::VERSION
        ));
        system_tray.add_quit("Quit");

        system_tray.run(true);
    });
}
