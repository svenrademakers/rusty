use crate::script_engine::*;
use std::any::Any;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum InterpreterType {
    Python,
}

impl std::fmt::Display for InterpreterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterType::Python => write!(f, "Python"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Script {
    pub key: ScriptKey,
    pub name: String,
    pub description: String,
    pub argument_type: Vec<ArgumentType>,
    pub argument_descriptions: Vec<String>,
    pub file: PathBuf,
}

impl Script {
    pub fn new(key: ScriptKey) -> Self {
        let mut script = Script::default();
        script.key = key;
        script
    }
}

pub type InterpreterArc = Arc<Mutex<dyn Interpreter + Send>>;
pub enum CallError {
    KeyNotPresent(ScriptKey),
    WrongArguments,
}

pub trait Interpreter {
    /// parse file and return the number of scripts found
    fn parse(
        &mut self,
        content: &[u8],
        file: &Path,
    ) -> Result<(usize, Vec<ParseError>), InterpreterError>;
    /// finish loading. update the keys with actual script keys and return script
    /// information.
    fn load(&mut self, keys: Vec<ScriptKey>) -> Result<Vec<Script>, InterpreterError>;

    fn call(&self, key: ScriptKey, args: &[Box<dyn Any>]) -> Result<(), CallError>;
}

pub fn parse_and_interpreter(
    interpreter: InterpreterArc,
) -> Result<(usize, Vec<ParseError>, InterpreterArc), ScriptEngineError> {
    match interpreter.lock() {
        Ok(mut data) => data
            .parse()
            .map_err(|e| ScriptEngineError::InterpretError(e.clone()))
            .map(|(count, errors)| (count, errors, interpreter.clone())),
        Err(e) => Err(ScriptEngineError::PoisonedMutex(e.to_string())),
    }
}

pub fn load(
    interpreter: InterpreterArc,
    keys: Vec<ScriptKey>,
) -> Result<Vec<Script>, ScriptEngineError> {
    match interpreter.lock() {
        Ok(mut data) => data
            .load(keys)
            .map_err(|e| ScriptEngineError::InterpretError(e.clone())),
        Err(e) => Err(ScriptEngineError::PoisonedMutex(e.to_string())),
    }
}

pub fn get_interpreter_for_file(file: &Path) -> Result<InterpreterArc, ScriptEngineError> {
    if file.extension().unwrap() == &OsString::from(PYTHON_EXTENSION) {
        Ok(Arc::new(Mutex::new(PyInterpreter::new(&file))))
    } else {
        Err(ScriptEngineError::InterpreterNotAvailable(
            file.extension().unwrap().to_os_string(),
        ))
    }
}
