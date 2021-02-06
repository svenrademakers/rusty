extern crate slotmap;
mod interpreter;
use crate::logging::*;
use interpreter::*;
pub use slotmap::*;
use std::boxed::Box;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::vec::Vec;

const PYTHON_EXTENSION: &str = "py";

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

#[derive(Debug)]
enum ScriptEngineError {
    ScriptKeyDoesNotExist(ScriptKey),
    NoInterpreterAvailable(InterpreterType),
}

impl std::fmt::Display for ScriptEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptEngineError::ScriptKeyDoesNotExist(key) => {
                write!(f, "{:?} does not exist.", key)
            }
            ScriptEngineError::NoInterpreterAvailable(x) => {
                write!(f, "{} interpreter is not loaded.", x)
            }
        }
    }
}

impl std::error::Error for ScriptEngineError {}

pub struct ScriptEngine {
    context: ScriptStore,
    interpreters: HashMap<InterpreterType, Box<dyn Interpreter>>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        ScriptEngine {
            context: ScriptStore::default(),
            interpreters: HashMap::new(),
        }
    }

    pub fn load(&mut self, scripts_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.load_bindings();

        for entry in std::fs::read_dir(scripts_path)? {
            if let Ok(valid) = entry {
                if let Some(x) = valid.path().extension().and_then(OsStr::to_str) {
                    match x {
                        PYTHON_EXTENSION => {return self.interpreters
                                        .get_mut(&InterpreterType::Python)
                                        .unwrap()
                                        .parse(&valid.path(), &mut self.context);},
                        _ => {
                            debug!("skip {:?} for parsing", valid.path());
                        }
                    }
                }
            } else {
                warn!("Skipping loading of {:?}", entry);
            }
        }

        Ok(())
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

    pub fn call(&self, script_key: ScriptKey, args: &[Argument]) -> Result<bool, Box<dyn Error>> {
        let interpreter_type = self
            .context
            .scripts
            .get(script_key)
            .ok_or(ScriptEngineError::ScriptKeyDoesNotExist(script_key))?;

        self.get_interpreter(*interpreter_type)?
            .call(script_key, args)
    }

    fn load_bindings(&mut self) {
        self.interpreters
            .insert(InterpreterType::Python, Box::new(PyInterpreter::new()));
    }

    fn get_interpreter(
        &self,
        interpreter_type: InterpreterType,
    ) -> Result<&Box<dyn Interpreter>, Box<dyn Error>> {
        match self.interpreters.get(&interpreter_type) {
            Some(x) => Ok(x),
            None => Err(Box::new(ScriptEngineError::NoInterpreterAvailable(
                interpreter_type,
            ))),
        }
    }
}
