extern crate app_dirs;
use app_dirs::*;

pub const APP_INFO: AppInfo = AppInfo {
    name: "Svensson",
    author: "Sven Rademakers < sven.rademakers@gmail.com > ",
};

pub const VERSION: &str = "asdfsd";
pub const BUILD_DATE: &str = "2020-12-1";

// pub fn get_settings_file() -> std::path::Path
// {
//  let wat = Path::new("wat");
//  wat
// }
// static SETTINGS_FILE: &Path = Path::new(&get_app_root(AppDataType::UserConfig, &APP_INFO));
