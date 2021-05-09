mod interpreter;
mod py_interpreter;

use crate::logging::*;

use interpreter::*;
use py_interpreter::*;
use std::{sync::mpsc::Receiver, time::Duration};
use threadpool::ThreadPool;

pub use slotmap::*;
use std::path::Path;
use std::{
    any::Any,
    boxed::Box,
    ffi::OsString,
    fs::DirEntry,
    sync::{mpsc::channel, Arc, Mutex},
};
use std::{path::PathBuf, vec::Vec};

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
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::Error => write!(f, "Not good!"),
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

impl<'a> std::error::Error for ScriptEngineError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptEngineCmd {
    Load { path: PathBuf },
    Call { key: ScriptKey },
}

pub struct ScriptEngine {
    scripts: SlotMap<ScriptKey, InterpreterArc>,
    names: SecondaryMap<ScriptKey, String>,
    description: SecondaryMap<ScriptKey, String>,
    argument_type: SecondaryMap<ScriptKey, Vec<ArgumentType>>,
    argument_descriptions: SecondaryMap<ScriptKey, Vec<String>>,
    files: SecondaryMap<ScriptKey, PathBuf>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        ScriptEngine {
            scripts: SlotMap::with_key(),
            names: SecondaryMap::new(),
            description: SecondaryMap::new(),
            argument_type: SecondaryMap::new(),
            argument_descriptions: SecondaryMap::new(),
            files: SecondaryMap::new(),
        }
    }

    fn pool_is_busy(pool: &ThreadPool) -> bool {
        pool.active_count() > 0 && pool.queued_count() > 0
    }

    fn receive_parse_res_or_print(
        recv: &Receiver<Result<(usize, Vec<ParseError>, InterpreterArc), ScriptEngineError>>,
    ) -> Result<(usize, Vec<ParseError>, InterpreterArc), ScriptEngineError> {
        let recv_res = recv.recv_timeout(Duration::from_secs(3));
        match recv_res {
            Ok(parse_result) => parse_result,
            Err(e) => {
                error!("{}", e);
                Err(ScriptEngineError::ErrorMessage("".to_string()))
            }
        }
    }

    pub fn load(&mut self, scripts_path: &Path) -> Result<Vec<ParseError>, ScriptEngineError> {
        let pool = ThreadPool::new(8);

        let files = get_files_of_dir(scripts_path)?;
        let rx_parse = parse_files(&files, &pool);

        let mut errors = Vec::new();
        let rx_load = self.load_files(&pool, files, rx_parse, &mut errors);
        pool.join();

        self.store_new_scripts(rx_load, scripts_path);
        Ok(errors)
    }

    fn store_new_scripts(
        &mut self,
        rx_load: Receiver<Result<Vec<Script>, ScriptEngineError>>,
        scripts_path: &Path,
    ) {
        for result in rx_load.recv().iter() {
            let scripts = result.as_ref().unwrap();
            for script in scripts {
                self.names.insert(script.key, script.name.clone());
                self.description
                    .insert(script.key, script.description.clone());
                self.argument_type
                    .insert(script.key, script.argument_type.clone());
                self.argument_descriptions
                    .insert(script.key, script.argument_descriptions.clone());
                self.files.insert(script.key, scripts_path.to_path_buf());
            }
        }
    }

    fn load_files(
        &mut self,
        pool: &ThreadPool,
        files: Vec<PathBuf>,
        rx_parse: Receiver<Result<(usize, Vec<ParseError>, InterpreterArc), ScriptEngineError>>,
        parse_errors: &mut Vec<ParseError>,
    ) -> Receiver<Result<Vec<Script>, ScriptEngineError>> {
        let mut received_parse = 0;
        let (tx_load, rx_load) = channel();
        while ScriptEngine::pool_is_busy(pool) && received_parse < files.len() {
            let result = ScriptEngine::receive_parse_res_or_print(&rx_parse);

            if let Ok((count, errors, interpreter)) = result {
                parse_errors.extend(errors);
                let keys = self.get_script_keys(count, interpreter.clone());
                let tx_load = tx_load.clone();
                pool.execute(move || {
                    let result = interpreter::load(interpreter, keys);
                    tx_load.send(result).unwrap();
                });
            }
            received_parse += 1;
        }
        rx_load
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
        script_key: ScriptKey,
        args: &[Box<dyn Any>],
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

fn parse_files(
    files: &Vec<PathBuf>,
    pool: &ThreadPool,
) -> Receiver<Result<(usize, Vec<ParseError>, InterpreterArc), ScriptEngineError>> {
    let (tx_parse, rx_parse) = channel();
    for i in 0..files.len() {
        let tx_parse = tx_parse.clone();
        let file = files.get(i).unwrap().clone();

        pool.execute(move || {
            let result =
                get_interpreter_for_file(&file).and_then(interpreter::parse_and_interpreter);
            tx_parse.send(result).unwrap();
        });
    }
    rx_parse
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
