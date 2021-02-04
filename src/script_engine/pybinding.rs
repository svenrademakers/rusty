extern crate pyo3;

use crate::script_engine::*;
use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyModule},
};

pub struct PyInterpreter {}
impl PyInterpreter {
    pub const fn new() -> Self {
        PyInterpreter {}
    }

    fn bind_functions(
        filename: &str,
        module_name: &str,
        contents: &str,
        script_store: &mut ScriptStore,
    ) -> ScriptKey {
        let key = script_store.scripts.insert(InterpreterType::Python);

        let gil = Python::acquire_gil();
        let py = gil.python();
        let module = PyModule::from_code(py, &contents, &filename, &module_name).unwrap();

        for key in module.dict().keys() {
            let obj = module.get(key.str().unwrap().to_str().unwrap()).unwrap();
            if obj.is_callable() {
                println!("{:?}{}", obj, obj.is_callable());
            }
        }
        key
    }
}

impl Interpreter for PyInterpreter {
    fn parse(&self, path: &Path, script_store: &mut ScriptStore) {
        if let Ok(contents) = std::fs::read_to_string(path) {
            let filename = path.file_name().and_then(OsStr::to_str).unwrap();
            let module = path.file_stem().and_then(OsStr::to_str).unwrap();
            let key = PyInterpreter::bind_functions(&filename, &module, &contents, script_store);
            script_store
                .files
                .insert(key, path.to_string_lossy().to_string());
        }
    }

    fn call(&self, script_key: ScriptKey, args: &[Argument]) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let py = PyInterpreter::new();
    }
}
