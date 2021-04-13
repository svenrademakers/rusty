use flaunch_core::logging::*;
use flaunch_ui::*;

fn main() {
    init_logging(LevelFilter::Debug).unwrap();
    unsafe { init();
        mainloop();
    }
}
