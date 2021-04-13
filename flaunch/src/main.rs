use flaunch_core::logging::*;
use flaunch_ui::*;
use flaunch_ui::root::ui::*;
use flaunch_core::app_meta::*;


fn main() {
    init_logging(LevelFilter::Debug).unwrap();
    unsafe {
        init(to_c_char(VERSION), to_c_char(BUILD_DATE));
        mainloop();
    }
 }
