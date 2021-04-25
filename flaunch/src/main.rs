use std::any::Any;

use flaunch_core::{logging::*, script_engine::*};
use flaunch_core::{settings::*, *};
use flaunch_ui::root::ui::*;
use flaunch_ui::*;
use script_engine::{KeyData, ScriptKey};
use system_tray::*;

static mut APPLICATION: FLaunchApplication = FLaunchApplication::new();

struct FLaunchApplication {
    scripts_engine: Option<ScriptEngine>,
    settings: Option<Settings<SettingKey>>,
}

impl FLaunchApplication {
    pub const fn new() -> Self {
        FLaunchApplication {
            scripts_engine: None,
            settings: None,
        }
    }

    unsafe extern "C" fn execute_script(script_key: u64) {
        if let Some(engine) = APPLICATION.get_script_engine() {
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

    pub fn init(&mut self) {
        let modules = load_flaunch_core();
        self.scripts_engine = Some(modules.0);
        self.settings = Some(modules.1);

        unsafe {
            init(
                to_c_char(app_meta::VERSION),
                to_c_char(app_meta::BUILD_DATE),
            );

            if let Some(engine) = &self.scripts_engine {
                for script in &engine.context.scripts {
                    add_script(
                        script.0.data().as_ffi(),
                        to_c_char(&engine.context.names[script.0]),
                        Some(FLaunchApplication::execute_script),
                    );
                }
            }
        }
    }

    pub fn mainloop(&mut self) {
        unsafe {
            debug!("starting mainloop.");
            mainloop();
        }
    }

    pub fn get_script_engine(&self) -> &Option<ScriptEngine> {
        &self.scripts_engine
    }
}

fn main() {
    unsafe {
        APPLICATION.init();
        APPLICATION.mainloop();
    }
}
