#[link(name = "nanocui")]
extern "C" {
    fn init();
    fn mainloop();
}

pub fn nanocui_init() {
    unsafe {
        init();
    }
}

pub fn nanocui_mainloop() {
    unsafe {
        mainloop();
    }
}
