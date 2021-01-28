mod app_meta;
mod logging;
extern crate app_dirs;

use app_meta::*;
use logging::*;

fn main() {
    if init_logging().is_err() {
        println!("Error: Could not init logging.");
        return;
    }

    info!("App:\t{}", APP_INFO.name);
    info!("Author: \t{}", APP_INFO.author);
    info!("Version:\t{} ({})", VERSION, BUILD_DATE);
    info!("--------------------------------------");
}
