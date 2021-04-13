use crate::script_engine::*;

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

pub trait Interpreter {
    fn parse(
        &mut self,
        filename: &Path,
        script_store: &mut ScriptStore,
    ) -> Result<(), Box<dyn Error>>;
    fn call(&self, script_key: ScriptKey, args: &[Argument]) -> Result<bool, Box<dyn Error>>;
}
