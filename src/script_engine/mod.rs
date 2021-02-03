extern crate slotmap;

pub use slotmap::*;
use std::boxed::Box;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::vec::Vec;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum BindingType {
    Python,
}

#[derive(Clone, PartialEq)]
pub enum Argument {
    Boolean(bool),
    Int(i64),
    Uint(u64),
    Float(f64),
    String(String),
    List(Box<Vec<Argument>>),
}

new_key_type! {pub struct ScriptKey;}

#[derive(Default)]
pub struct ScriptStore {
    pub scripts: SlotMap<ScriptKey, BindingType>,
    pub names: SecondaryMap<ScriptKey, String>,
    pub description: SecondaryMap<ScriptKey, String>,
    pub arguments: SecondaryMap<ScriptKey, Vec<Argument>>,
    pub argument_descriptions: SecondaryMap<ScriptKey, Vec<String>>,
    pub files: SecondaryMap<ScriptKey, String>,
    bindings: HashMap<BindingType, Box<dyn Binding>>,
}

impl ScriptStore {
    pub fn load(&mut self, scripts_path: &str) -> bool {
        for entry in std::fs::read_dir(scripts_path)
            .unwrap()
            .map(|ent| ent.as_ref().unwrap().path())
            .filter(|et| et.is_file())
        {
            if let Some(x) = entry.extension().and_then(OsStr::to_str) {
                match x {
                    ".py" => {
                        let key = self.scripts.insert(BindingType::Python);
                        self.files[key] = entry.to_string_lossy().to_string();
                    }
                    _ => {}
                }
            }
        }

        self.load_bindings();
        true
    }

    /// find key for given name.
    /// O(n)
    pub fn find(&self, name: &str) -> Option<ScriptKey> {
        for k in &self.names {
            if k.1 == name {
                return Some(k.0);
            }
        }
        None
    }

    pub fn call(&self, script_key: ScriptKey, args: &[Argument]) -> bool {
        let binding_type = self.scripts[script_key];
        if let Some(x) = self.bindings.get(&binding_type) {
            return x.call(script_key, args);
        }
        false
    }

    fn load_bindings(&mut self) {
        self.bindings
            .insert(BindingType::Python, Box::new(PyBinding::new()));
    }
}

pub trait Binding {
    fn parse(&self, script_store: &mut ScriptStore);
    fn call(&self, script_key: ScriptKey, args: &[Argument]) -> bool;
}

struct PyBinding {}
impl PyBinding {
    pub const fn new() -> Self {
        PyBinding {}
    }
}
impl Binding for PyBinding {
    fn parse(&self, script_store: &mut ScriptStore) {}

    fn call(&self, script_key: ScriptKey, args: &[Argument]) -> bool {
        false
    }
}
