extern crate slotmap;
mod pybinding;

use pybinding::*;
pub use slotmap::*;
use std::boxed::Box;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::vec::Vec;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum InterpreterType {
    Unkown,
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
struct ScriptStore {
    pub scripts: SlotMap<ScriptKey, InterpreterType>,
    pub names: SecondaryMap<ScriptKey, String>,
    pub description: SecondaryMap<ScriptKey, String>,
    pub arguments: SecondaryMap<ScriptKey, Vec<Argument>>,
    pub argument_descriptions: SecondaryMap<ScriptKey, Vec<String>>,
    pub files: SecondaryMap<ScriptKey, String>,
}

pub struct ScriptEngine {
    context: ScriptStore,
    bindings: HashMap<InterpreterType, Box<dyn Interpreter>>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        ScriptEngine {
            context: ScriptStore::default(),
            bindings: HashMap::new(),
        }
    }

    pub fn load(&mut self, scripts_path: &str) -> bool {
        self.load_bindings();

        for entry in std::fs::read_dir(scripts_path)
            .unwrap()
            .map(|ent| ent.as_ref().unwrap().path())
            .filter(|et| et.is_file())
        {
            if let Some(x) = entry.extension().and_then(OsStr::to_str) {
                match x {
                    "py" => {
                        self.bindings[&InterpreterType::Python].parse(&entry, &mut self.context);
                    }
                    _ => {}
                }
            } else {
                println!("wat{:?}", entry);
            }
        }

        true
    }

    /// find key for given name.
    /// O(n)
    pub fn find(&self, name: &str) -> Option<ScriptKey> {
        for k in &self.context.names {
            if k.1 == name {
                return Some(k.0);
            }
        }
        None
    }

    pub fn call(&self, script_key: ScriptKey, args: &[Argument]) -> bool {
        let binding_type = self.context.scripts[script_key];
        if let Some(x) = self.bindings.get(&binding_type) {
            return x.call(script_key, args);
        }
        false
    }

    fn load_bindings(&mut self) {
        self.bindings
            .insert(InterpreterType::Python, Box::new(PyInterpreter::new()));
    }
}

pub trait Interpreter {
    fn parse(&self, filename: &Path, script_store: &mut ScriptStore);
    fn call(&self, script_key: ScriptKey, args: &[Argument]) -> bool;
}
