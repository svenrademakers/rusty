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

pub struct Script {
    pub key: ScriptKey,
    pub name: String,
    pub description: String,
    pub argument_type: Vec<ArgumentType>,
    pub argument_descriptions: Vec<String>,
    pub file: PathBuf,
}

pub type ScriptInterpreterResult = (Vec<Script>, Vec<ParseError>);
pub type InterpreterArc = Arc<Mutex<dyn Interpreter + Send>>;
pub enum CallError {
    KeyNotPresent,
}

pub trait Interpreter {
    /// parse file and return the number of scripts found
    fn parse(&mut self) -> Result<usize, InterpreterError>;
    /// finish loading. update the keys with actual script keys and return script
    /// information.
    fn load(&mut self, keys: &[ScriptKey]) -> Result<ScriptInterpreterResult, InterpreterError>;

    fn call(&self, key: ScriptKey, args: &[Box<dyn Any>]) -> Result<(), CallError>;
}

pub fn parse_and_interpreter(
    interpreter: InterpreterArc,
) -> Result<(usize, InterpreterArc), ScriptEngineError> {
    match interpreter.lock() {
        Ok(mut data) => data
            .parse()
            .map_err(|e| ScriptEngineError::InterpretError(e.clone()))
            .map(|res| (res, interpreter.clone())),
        Err(e) => Err(ScriptEngineError::PoisonedMutex(e.to_string())),
    }
}

pub fn load(
    interpreter: InterpreterArc,
    keys: &[ScriptKey],
) -> Result<ScriptInterpreterResult, ScriptEngineError> {
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
