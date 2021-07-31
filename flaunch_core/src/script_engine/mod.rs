mod interpreter;
mod py_interpreter;
use crate::logging::*;

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
    call_map: HashMap<u64, Arc<dyn Callable>>,
}
unsafe impl Send for ScriptEngine {}

impl Default for ScriptEngine {
    fn default() -> Self {
        let (s, r) = watch::channel(ScriptChange::Deleted(0));

        ScriptEngine {
            script_sender: s,
            script_receiver: r,
            call_map: HashMap::new(),
        }
    }
}

impl ScriptEngine {
    pub fn observe(&self) -> watch::Receiver<ScriptChange> {
        self.script_receiver.clone()
    }

    pub async fn load(
        &mut self,
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
                    self.insert_callables(callables);
                    self.process_new_scripts(scripts);
                }
                complete => break,
            }
        }

        Ok(errors)
    }

    fn insert_callables(&mut self, callables: Vec<(u64, Arc<dyn Callable>)>) {
        for call in callables {
            self.call_map.insert(call.0, call.1);
        }
    }

    fn process_new_scripts(&mut self, scripts: Vec<Script>) {
        self.script_sender
            .send(ScriptChange::NewOrUpdated(scripts))
            .unwrap();
    }

    pub fn call(&self, script_key: u64, args: &[Box<dyn Any>]) -> Result<bool, ScriptEngineError> {
        if let Some(c) = self.call_map.get(&script_key) {
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

// #[derive(Debug, Default)]
// struct ScriptStore {
//     scripts: HashSet<u64>,
//     callables: Vec<(u64, Arc<dyn Callable>)>,
//     names: Vec<(u64, String)>,
//     description: Vec<(u64, String)>,
//     argument_type: Vec<(u64, Vec<ArgumentType>)>,
//     argument_descriptions: Vec<(u64, Vec<String>)>,
//     files: Vec<(u64, PathBuf)>,
// }

// impl ScriptStore {
//     /// requires `ScriptStore::sync` after calling this function.
//     /// store runs worst-case O(Log(n))
//     pub fn store_new_scripts(&mut self, scripts: &[Script]) {
//         for script in scripts {
//             let key = script.get_key();
//             if key.is_none() {
//                 error!("could not generate key");
//                 continue;
//             }

//             let key = key.unwrap();
//             store(self.scripts, key.clone(), script.call_context);
//             store(self.names, key.clone(), script.name);

//             store(self.description, key.clone(), script.description.clone());
//             store(
//                 self.argument_type,
//                 key.clone(),
//                 script.argument_type.clone(),
//             );
//             store(
//                 self.argument_descriptions,
//                 key.clone(),
//                 script.argument_descriptions.clone(),
//             );
//             store(files, key.clone(), script.file.to_path_buf());
//         }
//     }

//     fn store<K, V>(vec: &mut Vec<(K, V)>, key: K, val: V) {
//         if let Ok(index) = vec.binary_search_by_key(&key, |&(a, b)| b) {
//             vec[index] = (key, val);
//         } else {
//             vec.push((key, val));
//         }
//     }

//     async fn sync_vec<K, V>(vec: &mut Vec<(K, V)>, keys: &HashSet<u64>) {
//         vec.sort_by_cached_key(|&(a, b)| b);
//         //dedup and remove
//         // verify doc, dedup on sorted list should be fast
//         vec.dedup_by(|&(a, b)| {
//             // todo: verify if the last element will be removed if its stale
//             let remove = keys.contains(a.0);
//             remove || a.0 == b.0
//         });
//     }

//     pub fn sync(&mut self) {
//         Runtime::block_on(vec![
//             ScriptStore::sync_vec(&self.callables),
//             ScriptStore::sync_vec(&self.names),
//             ScriptStore::sync_vec(&self.description),
//             ScriptStore::sync_vec(&self.argument_type),
//             ScriptStore::sync_vec(&self.argument_descriptions),
//             ScriptStore::sync_vec(&self.files),
//         ]);
//     }
// }
