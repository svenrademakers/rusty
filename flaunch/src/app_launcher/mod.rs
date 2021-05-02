use std::sync::mpsc::{self, Receiver, Sender};

use flaunch_core::app_meta;
use fruitbasket::*;

pub use crate::system_tray::StatusBar;
pub use crate::system_tray::TStatusBar;

#[cfg(target_os = "linux")]
pub type AppLauncher = DummyContainer;
#[cfg(target_os = "macos")]
pub type AppLauncher = OsxLauncher;
#[cfg(target_os = "windows")]
pub type AppLauncher = win::WindowsStatusBar;

pub trait AppLauncherT {
    fn build(&mut self);
    fn set_resources();

    fn build_system_tray(&self) -> StatusBar;
    fn run(&mut self);
}
pub struct OsxLauncher {
    fruit_app: Option<FruitApp>,
    trampoline: Option<Trampoline>,
    tx: Sender<String>,
    rx: Receiver<String>,
}

impl OsxLauncher {
    pub fn new() -> Self {
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        OsxLauncher {
            fruit_app: None,
            trampoline: None,
            tx: tx,
            rx: rx,
        }
    }
}

impl AppLauncherT for OsxLauncher {
    fn build(&mut self) {
        let names = format!("{}{}", app_meta::APP_INFO.author, app_meta::APP_INFO.name);

        let mut trampoline =
            fruitbasket::Trampoline::new(app_meta::APP_NAME, app_meta::APP_NAME, &names);
        trampoline.version(app_meta::VERSION);
        trampoline.icon(app_meta::ICON);

        let nsapp = trampoline
            .build(fruitbasket::InstallDir::UserApplications)
            .unwrap();
        nsapp.set_activation_policy(fruitbasket::ActivationPolicy::Regular);

        self.trampoline = Some(trampoline);
        self.fruit_app = Some(nsapp);
    }

    fn set_resources() {}

    fn build_system_tray(&self) -> StatusBar {
        StatusBar::new(self.tx.clone(), app_meta::APP_NAME, app_meta::ICON)
    }

    fn run(&mut self) {
        self.fruit_app
            .as_mut()
            .unwrap()
            .run(fruitbasket::RunPeriod::Forever)
            .unwrap();
    }
}
