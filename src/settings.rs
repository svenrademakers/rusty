extern crate json;
use crate::app_meta;
use crate::logging;
use logging::{debug, error, info};

const SCRIPTS_DIR_KEY: &str = "script_directory";

lazy_static! {
    static ref APP_CONFIG: std::path::PathBuf =
        app_dirs::get_app_root(app_dirs::AppDataType::UserConfig, &app_meta::APP_INFO).unwrap();
    static ref CONF_FILENAME: String = {
        let mut config = APP_CONFIG.clone();
        config.push("config.json");
        config.to_string_lossy().to_string()
    };
}

fn load_defaults() -> std::collections::HashMap<String, String> {
    let mut dict = std::collections::HashMap::new();
    dict.insert(
        SCRIPTS_DIR_KEY.to_owned(),
        match std::env::current_dir() {
            Ok(s) => {
                let mut dir = s;
                dir.push("scripts");
                dir.to_string_lossy().to_string()
            }
            Err(e) => {
                error!("{:?}", e);
                "".to_string()
            }
        },
    );
    dict
}

pub struct Settings {
    settings: std::collections::HashMap<String, String>,
}

impl Settings {
    pub fn new() -> Self {
        let mut set = Settings {
            settings: load_defaults(),
        };

        set.load_setting_overrides();
        set
    }

    fn load_setting_overrides(&mut self) {
        if !std::path::Path::new(CONF_FILENAME.as_str()).exists() {
            info!("app config {}, not found", CONF_FILENAME.as_str());
            return;
        }

        if let Ok(json) = json::parse(&CONF_FILENAME) {
            for (key, value) in json.entries() {
                if let Some(s) = value.as_str() {
                    self.settings.insert(key.to_string(), s.to_string());
                } else {
                    debug!("key:{} val:{:?} not a string", key, value);
                }
            }
        } else {
            error!("parse error parsing {}", CONF_FILENAME.as_str());
        }
    }

    pub fn scripts_path(&self) -> Option<&String> {
        self.settings.get(SCRIPTS_DIR_KEY)
    }
}
