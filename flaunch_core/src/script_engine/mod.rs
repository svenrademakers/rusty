mod interpreter;
mod py_interpreter;

use crate::logging::*;

use futures::stream::FuturesUnordered;
pub use interpreter::Script;
use py_interpreter::*;
use std::ffi::CString;
use tokio::sync::broadcast::{channel, Receiver, Sender};

use futures::StreamExt;
use futures::{future::FutureExt, select};
pub use slotmap::*;
use std::path::Path;
use std::{any::Any, boxed::Box, ffi::OsString, fs::DirEntry, sync::Arc};

use std::{path::PathBuf, vec::Vec};

use self::interpreter::InterpreterArc;

const PYTHON_EXTENSION: &str = "py";

#[derive(Clone, PartialEq, Debug)]
pub enum ArgumentType {
    Boolean(String),
    Int(String),
    Uint(String),
    Float(String),
    String(String),
    List(String),
}

new_key_type! {pub struct ScriptKey;}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpreterError {
    Error,
    NotEnoughKeys { needed: usize, provided: usize },
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::Error => write!(f, "Not good!"),
            InterpreterError::NotEnoughKeys { needed, provided } => {
                write!(f, "Needed {} keys. Got {}", needed, provided)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptEngineError {
    ScriptKeyDoesNotExist(ScriptKey),
    InterpreterNotAvailable(OsString),
    MissingArguments(Vec<String>, usize),
    NoScriptsFound(PathBuf),
    InterpretError(InterpreterError),
    PoisonedMutex(String),
    ErrorMessage(String),
}
#[derive(Default, Clone, PartialEq, Debug)]
pub struct ParseError {
    pub filename: String,
    pub message: String,
    pub traceback: String,
}

impl<'a> std::fmt::Display for ScriptEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptEngineError::ScriptKeyDoesNotExist(key) => {
                write!(f, "{:?} does not exist.", key)
            }
            ScriptEngineError::InterpreterNotAvailable(x) => {
                write!(f, "no interpreter to load type {:?}.", x)
            }
            ScriptEngineError::MissingArguments(vec, len) => {
                write!(f, "missing arguments {:?} expected arguments={}", vec, len)
            }
            ScriptEngineError::NoScriptsFound(directory) => {
                write!(f, "no scripts found in {}", directory.to_string_lossy())
            }
            ScriptEngineError::InterpretError(error) => {
                write!(f, "{}", error)
            }
            ScriptEngineError::PoisonedMutex(e) => {
                write!(f, "{}", e)
            }
            ScriptEngineError::ErrorMessage(e) => {
                write!(f, "{}", e)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum ScriptChange {
    New(CScript),
    Deleted(u64),
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct CScript {
    pub key: u64,
    pub name: CString,
}

impl<'a> std::error::Error for ScriptEngineError {}

pub struct ScriptEngine {
    scripts: SlotMap<ScriptKey, InterpreterArc>,
    names: SecondaryMap<ScriptKey, String>,
    description: SecondaryMap<ScriptKey, String>,
    argument_type: SecondaryMap<ScriptKey, Vec<ArgumentType>>,
    argument_descriptions: SecondaryMap<ScriptKey, Vec<String>>,
    files: SecondaryMap<ScriptKey, PathBuf>,
    recv_change: Receiver<ScriptChange>,
    send_change: Sender<ScriptChange>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let (send, recv) = channel(64);
        ScriptEngine {
            scripts: SlotMap::with_key(),
            names: SecondaryMap::new(),
            description: SecondaryMap::new(),
            argument_type: SecondaryMap::new(),
            argument_descriptions: SecondaryMap::new(),
            files: SecondaryMap::new(),
            recv_change: recv,
            send_change: send,
        }
    }

    // pub fn get_receiver(&mut self) -> Receiver<ScriptChange> {
    //     self.recv_change.clone()
    // }

    pub async fn load(
        &mut self,
        scripts_path: &Path,
    ) -> Result<Vec<ParseError>, ScriptEngineError> {
        let mut parse_fut = FuturesUnordered::new();
        let mut load_fut = FuturesUnordered::new();

        let files = get_files_of_dir(scripts_path)?;
        if files.is_empty() {
            return Err(ScriptEngineError::NoScriptsFound(
                scripts_path.to_path_buf(),
            ));
        }

        for file in files {
            let parse_result = interpreter::read_and_parse_file(file).fuse();
            parse_fut.push(parse_result);
        }

        let mut errors: Vec<ParseError> = Vec::new();

        loop {
            select! {
                parse_res = parse_fut.select_next_some() => {
                    if let Ok((parse_ret, interpreter)) = parse_res {
                        errors.extend(parse_ret.1);
                        let keys = self.get_script_keys(parse_ret.0, interpreter.clone());
                        load_fut.push(interpreter::load(interpreter, keys).fuse());
                    }
                },
                load_res = load_fut.select_next_some() => {
                    if let Ok(scripts) = load_res {
                       self.store_new_scripts(&scripts);
                    }
                },
                complete => break,
            }
        }

        Ok(errors)
    }

    fn store_new_scripts(&mut self, scripts: &[Script]) {
        for script in scripts {
            self.names.insert(script.key, script.name.clone());
            self.description
                .insert(script.key, script.description.clone());
            self.argument_type
                .insert(script.key, script.argument_type.clone());
            self.argument_descriptions
                .insert(script.key, script.argument_descriptions.clone());
            self.files.insert(script.key, script.file.to_path_buf());
            self.send_script_change(script);
        }
    }

    fn send_script_change(&mut self, script: &Script) {
        let c_script = CScript {
            key: script.key.data().as_ffi(),
            name: CString::new(script.name.clone()).unwrap(),
        };
        self.send_change.send(ScriptChange::New(c_script)).unwrap();
    }

    fn get_script_keys(&mut self, result: usize, interpreter: InterpreterArc) -> Vec<ScriptKey> {
        let mut keys = Vec::new();
        for _ in 0..(result as i32) {
            let key = self.scripts.insert(interpreter.clone());
            keys.push(key);
        }
        keys
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

    pub fn call(
        &self,
        _script_key: ScriptKey,
        _args: &[Box<dyn Any>],
    ) -> Result<bool, ScriptEngineError> {
        //     let interpreter = self
        //         .scripts
        //         .get(script_key)
        //         .ok_or(ScriptEngineError::ScriptKeyDoesNotExist(script_key))?
        //         .clone();

        // self.return_on_invalid_arguments(&script_key, args.len())?;
        // self.get_interpreter(interpreter_type)?
        //     .call(script_key, args)
        Ok(false)
    }

    // fn return_on_invalid_arguments(
    //     &self,
    //     script_key: &ScriptKey,
    //     arg_len: usize,
    // ) -> Result<(), Box<dyn Error>> {
    //     let len = self.argument_type.get(*script_key).unwrap().len();
    //     if arg_len != len {
    //         return Err(Box::new(ScriptEngineError::MissingArguments(
    //             Vec::new(),
    //             len,
    //         )));
    //     }

    //     Ok(())
    // }
}

fn get_files_of_dir(dir: &Path) -> Result<Vec<PathBuf>, ScriptEngineError> {
    let filter = |d: std::io::Result<DirEntry>| match d {
        Ok(entry) => Some(entry.path()),
        Err(e) => {
            error!("{}", e);
            None
        }
    };

    let res = std::fs::read_dir(dir);
    match res {
        Err(x) => {
            error!("failed to get files in dir {:?} {}", dir, x);
            return Err(ScriptEngineError::NoScriptsFound(dir.to_path_buf()));
        }
        Ok(dir) => Ok(dir.filter_map(filter).collect()),
    }
}
