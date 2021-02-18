extern crate json;
use crate::logging;

pub use json::JsonValue;

use logging::{error, warn};
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::Copy;

pub type KeyWithDefault<Key> = (Key, &'static str, JsonValue);
pub type SettingsReloaded<T> = dyn Fn(&T);

pub struct Settings<Key>
where
    Key: Eq + Hash + Copy,
{
    settings: HashMap<Key, JsonValue>,
    mapping: HashMap<&'static str, Key>,
}

impl<Key> Settings<Key>
where
    Key: Eq + Hash + Copy,
{
    pub fn new(key_mapping: &[KeyWithDefault<Key>]) -> Self {
        let mut set = Settings::<Key> {
            settings: HashMap::new(),
            mapping: HashMap::new(),
        };

        for (key, json_key, default) in key_mapping {
            set.settings.insert(*key, default.clone());
            set.mapping.insert(json_key, *key);
        }
        set
    }

    pub fn load(&mut self, settings_file: &str) {
        self.from_json(&settings_file);
    }

    // pub fn set_str(&mut self, setting: Key, value: String) {
    //     self.settings[&setting] = json::JsonValue::String(value);
    // }

    pub fn get_str(&self, setting: Key) -> Option<&str> {
        self.settings.get(&setting).map(|x| x.as_str())?
    }

    // pub fn on_config_reload(&mut self, callback: &'a SettingsReloaded<Key>) {
    //     self.observers.push(callback);
    // }

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
