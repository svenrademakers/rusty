use crate::app_launcher::AppLauncherT;

pub struct WindowsLauncher {}

impl WindowsLauncher {
    pub fn new() -> Self {
        WindowsLauncher {}
    }
}

impl AppLauncherT for WindowsLauncher {
    fn set_resources() {}
}
