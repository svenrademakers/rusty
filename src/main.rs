mod application_data;
mod logging;
extern crate app_dirs;

use application_data::*;
use logging::*;
use std::path::Path;

fn main() {
    if init_logging().is_err() {
        println!("Error: Could not init logging.");
        return;
    }

    info!("App:\t{}", APP_INFO.name);
    info!("Author: \t{}", APP_INFO.author);
    info!("Version:\t{} ({})", VERSION, BUILD_DATE);
    info!("--------------------------------------");

    let p = Path::new("sdf");
}
