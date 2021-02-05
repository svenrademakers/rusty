extern crate pyo3;

use crate::script_engine::*;
pub use pyo3::{prelude::*, types::PyModule};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum InterpreterType {
    Python,
}

impl std::fmt::Display for InterpreterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Python => write!(f, "Python"),
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

pub struct PyInterpreter<'a> {
    callables: SparseSecondaryMap<ScriptKey, &'a PyAny>,
    _gil: GILGuard,
    py: Python<'a>,
}

impl<'a> PyInterpreter<'a> {
    pub fn new() -> Self {
        PyInterpreter {
            callables: SparseSecondaryMap::new(),
            _gil: Python::acquire_gil(),
            py: unsafe { Python::assume_gil_acquired() },
        }
    }

    fn bind_to_python(&self, contents: &str, filename: &str, module_name: &str) -> Vec<&'a PyAny> {
        let mut objects = Vec::new();
        let module = PyModule::from_code(self.py, &contents, &filename, &module_name).unwrap();

        for obj in module.dict().keys() {
            let name = obj.str().unwrap().to_str().unwrap();
            let obj = module.get(&name).unwrap();
            if obj.is_callable() {
                objects.push(obj);
            }
        }
        objects
    }
}

impl<'a> Interpreter for PyInterpreter<'a> {
    fn parse(&mut self, path: &Path, script_store: &mut ScriptStore) -> Result<(), Box<dyn Error>> {
        if let Ok(contents) = std::fs::read_to_string(path) {
            let filename = path.file_name().and_then(OsStr::to_str).unwrap();
            let module_name = path.file_stem().and_then(OsStr::to_str).unwrap();

            for obj in self.bind_to_python(&contents, &filename, &module_name) {
                let key = script_store.scripts.insert(InterpreterType::Python);
                self.callables.insert(key, obj);
                script_store
                    .names
                    .insert(key, obj.str().unwrap().to_string_lossy().to_string());

                script_store
                    .files
                    .insert(key, path.to_string_lossy().to_string());
            }
        }

        Ok(())
    }

    fn call(&self, script_key: ScriptKey, _args: &[Argument]) -> Result<bool, Box<dyn Error>> {
        println!("TEST");
        if let Some(x) = self.callables.get(script_key) {
            if let Ok(res) = x.call0() {
                println!("Called successfully {}", res);
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_simple_func() {
        let interpreter = PyInterpreter::new();
        let simple_func = "def awesome_func():\n\tprint(\"hello!\")";

        let func = interpreter.bind_to_python(&simple_func, "test.py", "test");
        assert_eq!("awesome_func", func[0]);
    }
}
