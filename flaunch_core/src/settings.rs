extern crate json;
use crate::app_meta;
use crate::logging;
pub use json::JsonValue;

use logging::*;
use std::hash::Hash;
use std::marker::Copy;
use std::path::PathBuf;
use std::{cmp::Eq, collections::HashMap};
use tokio::sync::watch;

pub type KeyWithDefault<Key> = (Key, &'static str, JsonValue);

pub struct Settings<Key>
where
    Key: Eq + Hash + Copy + Clone,
{
    /// actual map with settings. value should always contain
    /// a valid value. on load value is insterted with a default
    // value.
    settings: HashMap<Key, JsonValue>,
    /// mapping from textual json key to rust enum key
    mapping: HashMap<&'static str, Key>,
    channel: (
        watch::Sender<SettingsChanged>,
        watch::Receiver<SettingsChanged>,
    ),
}
pub struct SettingsChanged(PathBuf);

impl<Key> Settings<Key>
where
    Key: Eq + Hash + Copy + Clone,
{
    pub fn new(key_mapping: &[KeyWithDefault<Key>]) -> Self {
        let mut set = Settings::<Key> {
            settings: HashMap::new(),
            mapping: HashMap::new(),
            channel: watch::channel(SettingsChanged(PathBuf::new())),
        };

        for (key, json_key, default) in key_mapping {
            set.settings.insert(*key, default.clone());
            set.mapping.insert(json_key, *key);
        }
        set
    }

    pub fn observe(&self) -> watch::Receiver<SettingsChanged> {
        self.channel.1.clone()
    }

    pub fn load(&mut self) {
        let settings_file = master_settings();
        debug!("master_settings {}", settings_file.to_string_lossy());
        if settings_file.exists() {
            self.from_json(&settings_file.to_string_lossy().to_string());
        }
    }

    pub fn reset(&self) {
        std::fs::remove_file(master_settings()).unwrap();
    }

    pub fn set_str(&mut self, setting: Key, value: String) {
        if let Some(val) = self.settings.get_mut(&setting) {
            if val.is_string() {
                *val = json::JsonValue::String(value);
            }
        }
    }

    pub fn get_str(&self, setting: Key) -> Option<&str> {
        self.settings.get(&setting).map(|x| x.as_str())?
    }

    fn from_json(&mut self, settings_file: &str) {
        if let Ok(contents) = std::fs::read_to_string(settings_file) {
            if let Ok(json) = json::parse(contents.as_str()) {
                for (json_key, json_value) in json.entries() {
                    if let Some(key) = self.mapping.get(json_key) {
                        if let Some(val) = self.settings.get_mut(key) {
                            *val = json_value.clone();
                        }
                    } else {
                        warn!("setting key {:?} not configured", json_key);
                    }
                }
            } else {
                error!("parse error parsing {}", settings_file);
            }
        } else {
            warn!("app config {}, not found", settings_file);
        }
    }
}

pub fn master_settings() -> std::path::PathBuf {
    let mut settings_file =
        app_dirs::get_app_root(app_dirs::AppDataType::UserConfig, &app_meta::APP_INFO).unwrap();
    settings_file.push("config.json");
    settings_file
}
