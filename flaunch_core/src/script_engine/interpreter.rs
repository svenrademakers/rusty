use futures::lock::Mutex;
use tokio::fs;

use crate::script_engine::*;
use std::any::Any;

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

pub type ParseResult = (usize, Vec<ParseError>);
pub trait Interpreter {
    /// parse file and return the number of scripts found
    fn parse(&mut self, content: &[u8], file: &Path) -> Result<ParseResult, InterpreterError>;
    /// finish loading. update the keys with actual script keys and return script
    /// information.
    fn load(&mut self, keys: Vec<ScriptKey>) -> Result<Vec<Script>, InterpreterError>;

    fn call(&self, key: ScriptKey, args: &[Box<dyn Any>]) -> Result<(), CallError>;
}

pub async fn read_and_parse_file(
    file: PathBuf,
) -> Result<(ParseResult, InterpreterArc), ScriptEngineError> {
    let content = fs::read(&file).await.unwrap();
    let interpreter = create_interpreter_for_file(&file)?;
    let result = interpreter::parse(&interpreter, &content, &file).await?;
    Ok((result, interpreter.clone()))
}

async fn parse(
    interpreter: &InterpreterArc,
    content: &[u8],
    file: &Path,
) -> Result<ParseResult, ScriptEngineError> {
    let mut inter = interpreter.lock().await;
    (*inter)
        .parse(content, &file)
        .map_err(|e| ScriptEngineError::InterpretError(e.clone()))
}

pub async fn load(
    interpreter: InterpreterArc,
    keys: Vec<ScriptKey>,
) -> Result<Vec<Script>, ScriptEngineError> {
    let mut inter = interpreter.lock().await;
    (*inter)
        .load(keys)
        .map_err(|e| ScriptEngineError::InterpretError(e.clone()))
}

pub fn create_interpreter_for_file(file: &Path) -> Result<InterpreterArc, ScriptEngineError> {
    if file.extension().unwrap() == &OsString::from(PYTHON_EXTENSION) {
        Ok(Arc::new(Mutex::new(PyInterpreter::new())))
    } else {
        Err(ScriptEngineError::InterpreterNotAvailable(
            file.extension().unwrap().to_os_string(),
        ))
    }
}
