extern crate pyo3;

use crate::script_engine::*;
pub use pyo3::{prelude::*, types::PyModule};

new_key_type! {pub struct ModuleKey;}

pub struct PyInterpreter<'a> {
    callables: SparseSecondaryMap<ScriptKey, &'a PyAny>,
    py: Python<'a>,
}

impl<'a> PyInterpreter<'a> {
    pub fn new() -> Self {
        PyInterpreter {
            callables: SparseSecondaryMap::new(),
            py: unsafe { Python::assume_gil_acquired() },
        }
    }

    fn bind_to_python(&self, contents: &str, filename: &str, module_name: &str) -> Vec<&'a PyAny> {
        let mut objects = Vec::new();
        let _gill = Python::acquire_gil();
        let module = PyModule::from_code(self.py, &contents, &filename, &module_name).unwrap();

        for obj in module.dict().keys() {
            let obj = module.get(obj.str().unwrap().to_str().unwrap()).unwrap();
            if obj.is_callable() {
                objects.push(obj);
            }
        }
        objects
    }
}

impl<'a> Interpreter for PyInterpreter<'a> {
    fn parse(&mut self, path: &Path, script_store: &mut ScriptStore) {
        if let Ok(contents) = std::fs::read_to_string(path) {
            let filename = path.file_name().and_then(OsStr::to_str).unwrap();
            let module_name = path.file_stem().and_then(OsStr::to_str).unwrap();

            for obj in self.bind_to_python(&contents, &filename, &module_name) {
                let key = script_store.scripts.insert(InterpreterType::Python);
                self.callables.insert(key, obj);

                script_store
                    .files
                    .insert(key, path.to_string_lossy().to_string());
            }
        }
    }

    fn call(&self, script_key: ScriptKey, _args: &[Argument]) -> bool {
        let _gill = Python::acquire_gil();
        if let Some(x) = self.callables.get(script_key) {
            if let Ok(res) = x.call0() {
                println!("Called successfully {}", res);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_simple_func() {
        let interpreter = PyInterpreter::new();
        let simple_func = "def awesome_func():\n\tprint(\"hello!\")";

        let wat = interpreter.bind_to_python(&simple_func, "test.py", "test");
        assert_ne!(wat.len(), 0);
    }
}
