extern crate json;
use crate::logging;

pub use json::JsonValue;

use logging::{error, info, warn};
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::Copy;
use std::path::Path;
use std::path::PathBuf;

use std::vec::Vec;

pub type KeyWithDefault<Key> = (Key, &'static str, JsonValue);
pub type SettingsReloaded<T> = dyn Fn(&T);

pub struct Settings<'a, Key>
where
    Key: Eq + Hash + Copy,
{
    settings: HashMap<Key, JsonValue>,
    mapping: HashMap<&'static str, Key>,
    observers: Vec<&'a SettingsReloaded<Key>>,
    settings_file: &'static str,
}

impl<'a, Key> Settings<'a, Key>
where
    Key: Eq + Hash + Copy,
{
    pub fn new(settings_file: &'static str, key_mapping: &[KeyWithDefault<Key>]) -> Self {
        let mut set = Settings::<Key> {
            settings: HashMap::new(),
            mapping: HashMap::new(),
            observers: Vec::new(),
            settings_file: settings_file,
        };
        json::object!();

        for (key, json_key, default) in key_mapping {
            set.settings.insert(*key, default.clone());
            set.mapping.insert(json_key, *key);
        }
        set.from_json();
        set
    }

    pub fn get(&self, setting: Key) -> Option<&JsonValue> {
        self.settings.get(&setting)
    }

    pub fn get_str(&self, setting: Key) -> Option<&str> {
        self.settings.get(&setting).map(|x| x.as_str())?
    }

    pub fn on_config_reload(&mut self, callback: &'a SettingsReloaded<Key>) {
        self.observers.push(callback);
    }

    fn from_json(&mut self) {
        if let Ok(contents) = std::fs::read_to_string(self.settings_file) {
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
                error!("parse error parsing {}", self.settings_file);
            }
        } else {
            warn!("app config {}, not found", self.settings_file);
        }
    }
}
