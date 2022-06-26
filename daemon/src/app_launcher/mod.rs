use flaunch_core::app_meta;
use futures::channel::mpsc::Sender;

pub use crate::system_tray::StatusBar;
pub use crate::system_tray::TStatusBar;

#[cfg(target_os = "linux")]
pub type AppLauncher = DummyContainer;
#[cfg(target_os = "macos")]
mod osx_launcher;
#[cfg(target_os = "macos")]
pub type AppLauncher = osx_launcher::OsxLauncher;

#[cfg(target_os = "windows")]
mod win_launcher;
#[cfg(target_os = "windows")]
pub type AppLauncher = win_launcher::WindowsLauncher;

pub trait AppLauncherT {
    fn set_resources();

    fn build_system_tray(&self, sender: Sender<String>) -> StatusBar {
        StatusBar::new(sender, app_meta::APP_NAME, app_meta::ICON)
    }

    fn configure_url_scheme(scheme: &str, description: &str);
}

pub struct DummyContainer {}
impl DummyContainer {
    pub fn new() -> Self {
        DummyContainer {}
    }
}

impl AppLauncherT for DummyContainer {
    fn set_resources() {}
    fn configure_url_scheme(_scheme: &str, _description: &str) {}
}
