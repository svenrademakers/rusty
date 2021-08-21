mod alias_interpreter;
mod interpreter;
mod py_interpreter;
use crate::file_watcher;
use crate::logging::*;

use futures::lock::Mutex;
use futures::select;
use futures::stream::FuturesUnordered;
use futures::FutureExt;
use futures::StreamExt;
pub use interpreter::Script;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;
use std::{any::Any, boxed::Box, ffi::OsString, fs::DirEntry};
use tokio::sync::watch;
use tokio::sync::watch::Receiver;
use tokio::sync::watch::Sender;

use std::{path::PathBuf, vec::Vec};

use self::interpreter::Callable;
use self::interpreter::ParseError;

#[derive(Clone, PartialEq, Debug)]
pub enum ArgumentType {
    Boolean(String),
    Int(String),
    Uint(String),
    Float(String),
    String(String),
    List(String),
}

#[derive(Debug, Clone)]
pub enum ScriptChange {
    NewOrUpdated(Vec<Script>),
    Deleted(u64),
}

#[derive(Debug)]
pub enum ScriptController {
    Load(PathBuf),
    Call(u64, Vec<Box<dyn Any>>),
}

unsafe impl Send for ScriptChange {}
unsafe impl Send for ScriptController {}

#[derive(Debug)]
pub struct ScriptEngine {
    script_sender: Sender<ScriptChange>,
    script_receiver: Receiver<ScriptChange>,
    call_map: Mutex<HashMap<u64, Arc<dyn Callable>>>,
    scripts: Mutex<HashMap<u64, Script>>,
    file_lookup: Mutex<HashMap<PathBuf, u64>>,
}

impl ScriptEngine {
    fn new(
        channel: (
            tokio::sync::mpsc::Sender<ScriptChange>,
            tokio::sync::mpsc::Receiver<ScriptChange>,
        ),
    ) -> Self {
        ScriptEngine {
            script_sender: channel.0,
            script_receiver: channel.1,
            call_map: Mutex::new(HashMap::new()),
            scripts: Mutex::new(HashMap::new()),
            file_lookup: Mutex::new(HashMap::new()),
        }
    }

    pub fn observe(&self) -> watch::Receiver<ScriptChange> {
        self.script_receiver.clone()
    }

    #[cfg(target_family = "unix")]
    pub async fn load_aliases(&self) -> Vec<ParseError> {
        use self::alias_interpreter::AliasInterpreter;

        let (scripts, callables, err) = AliasInterpreter::load();
        futures::join!(
            self.insert_callables(callables),
            self.process_new_scripts(scripts)
        );
        err
    }

    pub async fn load_path(
        &self,
        scripts_path: &Path,
    ) -> Result<Vec<ParseError>, ScriptEngineError> {
        let files = get_files_of_dir(scripts_path)?;
        if files.is_empty() {
            return Err(ScriptEngineError::NoScriptsFound(
                scripts_path.to_path_buf(),
            ));
        }

        let mut parse_fut = FuturesUnordered::new();
        for file in files {
            info!("{}= loading {}", module_path!(), file.to_string_lossy());
            let parse_task = interpreter::read_and_parse_file(file).fuse();
            parse_fut.push(parse_task);
        }

        let mut errors: Vec<ParseError> = Vec::new();

        loop {
            select! {
                parse_res = parse_fut.select_next_some() => {
                    let (scripts, callables, err) = parse_res;
                    errors.extend(err);
                    // todo spawn on new task?
                    futures::join!(
                        self.insert_callables(callables),
                        self.process_new_scripts(scripts)
                    );
                }
                complete => break,
            }
        }
        Ok(errors)
    }

    async fn insert_callables(&self, callables: Vec<(u64, Arc<dyn Callable>)>) {
        let mut callmap = self.call_map.lock().await;
        for call in callables {
            callmap.insert(call.0, call.1);
        }
    }

    async fn process_new_scripts(&self, scripts: Vec<Script>) {
        self.script_sender
            .send(ScriptChange::NewOrUpdated(scripts.clone()))
            .unwrap();

        let mut self_scripts = self.scripts.lock().await;
        let mut file_lookup = self.file_lookup.lock().await;
        for script in scripts.into_iter() {
            file_lookup.insert(script.file.clone(), script.get_key().unwrap());
            self_scripts.insert(script.get_key().unwrap(), script);
        }
    }

    pub async fn call(
        &self,
        script_key: u64,
        args: &[Box<dyn Any>],
    ) -> Result<bool, ScriptEngineError> {
        let callmap = self.call_map.lock().await;
        if let Some(c) = callmap.get(&script_key) {
            info!("{}= Calling script:{}", module_path!(), script_key);
            self.return_on_invalid_arguments(&script_key, args.len())?;
            c.call(script_key, args).unwrap();
            return Ok(true);
        } else {
            return Err(ScriptEngineError::ScriptKeyDoesNotExist(script_key));
        }
    }

    fn return_on_invalid_arguments(
        &self,
        _script_key: &u64,
        _arg_len: usize,
    ) -> Result<(), ScriptEngineError> {
        // let len = self.argument_type.get(*script_key).unwrap().len();
        // if arg_len != len {
        //     return Err(Box::new(ScriptEngineError::MissingArguments(
        //         Vec::new(),
        //         len,
        //     )));
        // }

        Ok(())
    }
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

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptEngineError {
    ScriptKeyDoesNotExist(u64),
    InterpreterNotAvailable(OsString),
    MissingArguments(Vec<String>, usize),
    NoScriptsFound(PathBuf),
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
        }
    }
}
