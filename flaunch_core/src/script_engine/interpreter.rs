use crate::script_engine::*;
use std::boxed::Box;
use std::hash::Hasher;
use std::{any::Any, collections::hash_map::DefaultHasher, hash::Hash};

use super::py_interpreter::PyInterpreter;
#[derive(Hash, Debug, Clone)]
pub enum InterpreterType {
    Python,
}

pub trait Callable: Debug {
    fn call(&self, key: u64, args: &[Box<dyn Any>]) -> Result<(), CallError>;
}

/// Result structure containing found script details.
/// Returned as part of the `Interpreter::parse` function
#[derive(Debug, Clone)]
pub struct Script {
    /// required field
    pub name: String,
    pub description: String,
    pub argument_type: Vec<ArgumentType>,
    pub argument_descriptions: Vec<String>,
    pub file: PathBuf,
    pub interpreter_type: InterpreterType,
}
unsafe impl Send for Script {}

impl Script {
    pub fn new(name: String, interpreter_type: InterpreterType) -> Script {
        Script {
            name: name,
            description: String::default(),
            argument_type: Vec::new(),
            argument_descriptions: Vec::new(),
            file: PathBuf::new(),
            interpreter_type: interpreter_type,
        }
    }

    pub fn get_key(&self) -> Option<u64> {
        if self.name.is_empty() {
            return None;
        }

        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        self.file.hash(&mut hasher);
        self.interpreter_type.hash(&mut hasher);
        Some(hasher.finish())
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct ParseError {
    pub filename: String,
    pub message: String,
    pub traceback: String,
}

#[derive(Debug)]
pub enum CallError {
    KeyNotPresent(u64),
    WrongArguments,
}

pub type ParseResult = (Vec<Script>, Vec<(u64, Arc<dyn Callable>)>, Vec<ParseError>);

pub trait Interpreter {
    /// Parses content of a given file and returns a list of found scripts.
    /// This function should be dumb and straight forward.
    /// The script engine will figure out itself the diff and update accordingly.
    ///
    /// # Hashing
    /// Hashes are the main keys to index scripts in the system.
    /// Implementors need to make sure to create unique and deterministic hashes
    /// that dont collide with other scripts. This includes scripts created by
    /// other interpreters.
    /// Adviced is to add a key unique to this interpreter to the hash.
    ///
    /// # call_context
    /// behavior
    /// `Vec<ParseError>` contains a list of parse errors found by the interpreter runtime
    fn parse(&self, content: &[u8], file: &Path) -> ParseResult;
}

pub async fn read_and_parse_file(file: PathBuf) -> ParseResult {
    let content = std::fs::read(&file).unwrap();
    match select_interpreter_for_file(&file) {
        Ok(it) => it.parse(&content, &file),
        Err(e) => {
            debug!("{}", e);
            ParseResult::default()
        }
    }
}

pub fn select_interpreter_for_file(
    file: &Path,
) -> Result<&'static dyn Interpreter, ScriptEngineError> {
    let file_ext = file.extension().unwrap().to_str().unwrap();

    match file_ext {
        "py" => Ok(&PYINTERPRETER),
        _ => Err(ScriptEngineError::InterpreterNotAvailable(
            file.extension().unwrap().to_os_string(),
        )),
    }
}

static PYINTERPRETER: PyInterpreter = PyInterpreter {};
