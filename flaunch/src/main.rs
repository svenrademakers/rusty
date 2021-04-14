use flaunch_core::*;
use flaunch_core::{logging::*, script_engine::Key, script_engine::ScriptEngine};
use flaunch_ui::root::ui::*;
use flaunch_ui::*;
use script_engine::{KeyData, ScriptKey};

// static mut engine_cached: Option<ScriptEngine> = None;

extern "C" fn clicked(script_key: u64) {
    //     println!("script pressed {}", script_key);
    //     let key = ScriptKey::from(KeyData::from_ffi(script_key));
    //     engine_cached.unwrap().call(key, &Vec::new());
}

fn main() {
    let modules = load_flaunch_core();
    let script_engine = modules.0;
    let settings = modules.1;

    unsafe {
        init(
            to_c_char(app_meta::VERSION),
            to_c_char(app_meta::BUILD_DATE),
        );

        for script in script_engine.context.scripts {
            add_script(
                script.0.data().as_ffi(),
                to_c_char(&script_engine.context.names[script.0]),
                Some(clicked),
            );
        }
        mainloop();
    }
}
