use std::{any::TypeId, collections::HashMap, path::PathBuf, process::Command, sync::Arc};

use regex::Regex;

use crate::script_engine::ArgumentType;

use super::{
    interpreter::{self, CallError, Callable, InterpreterType},
    Script,
};

#[derive(Debug)]
pub struct AliasInterpreter {
    aliases: HashMap<u64, String>,
}

impl AliasInterpreter {
    pub fn load() -> interpreter::ParseResult {
        let mut aliases = AliasInterpreter {
            aliases: HashMap::new(),
        };

        let mut scripts = Vec::new();
        let errors = Vec::new();

        let raw = Command::new("aliases").output().unwrap();
        let re = Regex::new(r"alias\s+(.*)=").unwrap();
        for cap in re.captures_iter(&String::from_utf8_lossy(&raw.stdout)) {
            let alias = &cap[1];
            let script = Script {
                name: alias.to_string(),
                description: String::new(),
                argument_type: vec![ArgumentType::String("Arguments".to_string())],
                argument_descriptions: Vec::new(),
                file: PathBuf::new(),
                interpreter_type: InterpreterType::Alias,
            };

            let key = script.get_key().unwrap();
            scripts.push(script);
            aliases.aliases.insert(key, alias.to_string());
        }

        let mut callables: Vec<(u64, Arc<dyn Callable>)> = Vec::new();
        let arc = Arc::new(aliases);
        for script in &scripts {
            callables.push((script.get_key().unwrap(), arc.clone()));
        }

        (scripts, callables, errors)
    }
}

impl Callable for AliasInterpreter {
    fn call(
        &self,
        key: u64,
        args: &[Box<dyn std::any::Any>],
    ) -> Result<(), interpreter::CallError> {
        if let Some(val) = self.aliases.get(&key) {
            if args.is_empty() || args[0].type_id() != TypeId::of::<String>() {
                return Err(CallError::WrongArguments);
            }
            let arg = args[0].downcast_ref::<String>().unwrap();
            let cmd = format!("{} {}", val, arg);
            match Command::new(cmd).status() {
                Ok(_) => Ok(()),
                // Ok(exit_status) => Err(CallError::NotSuccessful(exit_status.to_string())),
                Err(x) => Err(CallError::NotSuccessful(x.to_string())),
            }
        } else {
            Err(CallError::KeyNotPresent(key))
        }
    }
}
