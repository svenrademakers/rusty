use flaunch_core::logging::*;
use flaunch_core::*;
use flaunch_ui::*;
use flaunch_ui::root::ui::*;

fn main() {
    let modules = load_flaunch_core();
    let script_engine = modules.0;
    let settings = modules.1;

    unsafe {
        init(to_c_char(app_meta::VERSION), to_c_char(app_meta::BUILD_DATE));
        for script_name in script_engine.context.names {
            add_script(to_c_char(&script_name.1));
        }
        mainloop();
    }
 }
