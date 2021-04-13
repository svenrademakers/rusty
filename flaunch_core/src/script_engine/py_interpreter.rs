extern crate pyo3;
use crate::script_engine::interpreter::*;
use crate::script_engine::*;

pub use pyo3::{
    prelude::*,
    types::{PyModule, PyTuple},
};

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

    fn bind_to_python(
        &self,
        contents: &str,
        filename: &str,
        module_name: &str,
    ) -> Vec<(String, &'a PyAny)> {
        let mut objects = Vec::new();
        let module = PyModule::from_code(self.py, &contents, &filename, &module_name).unwrap();
        for obj in module.dict().keys() {
            let name = obj.str().unwrap().to_str().unwrap().to_string();
            let obj = module.get(&name).unwrap();

            if obj.is_callable() {
                objects.push((name, obj));
            }
        }
        objects
    }

    pub fn get_arguments(py_any: &PyAny) -> Vec<Argument> {
        let mut args = Vec::new();
        for var_name in py_any.getattr("__code__").unwrap().getattr("co_varnames") {
            args.push(Argument::String(var_name.to_string()));
        }

        args
    }
}

impl<'a> Interpreter for PyInterpreter<'a> {
    fn parse(&mut self, path: &Path, script_store: &mut ScriptStore) -> Result<(), Box<dyn Error>> {
        if let Ok(contents) = std::fs::read_to_string(path) {
            let filename = path.file_name().and_then(OsStr::to_str).unwrap();
            let module_name = path.file_stem().and_then(OsStr::to_str).unwrap();

            for (name, obj) in self.bind_to_python(&contents, &filename, &module_name) {
                let key = script_store.scripts.insert(InterpreterType::Python);
                self.callables.insert(key, obj);
                script_store.names.insert(key, name);
                script_store
                    .arguments
                    .insert(key, PyInterpreter::get_arguments(obj));
                script_store
                    .files
                    .insert(key, path.to_string_lossy().to_string());

                if obj.hasattr("__doc__")? {
                    let doc = obj.getattr("__doc__").unwrap().to_string();
                    script_store.description.insert(key, doc);
                }
            }
        }

        Ok(())
    }

    fn call(&self, script_key: ScriptKey, args: &[Argument]) -> Result<bool, Box<dyn Error>> {
        if let Some(x) = self.callables.get(script_key) {
            if args.len() == 0 {
                x.call0().unwrap();
            } else {
                let mut py_arguments = Vec::new();
                for arg in args {
                    match arg {
                        Argument::String(x) => py_arguments.push(x),
                        _ => {}
                    }
                }
                let py_args = PyTuple::new(self.py, py_arguments);
                x.call1(py_args).unwrap();
            }
        }
        Ok(true)
    }
}

  unsafe impl<'a> std::marker::Send for PyInterpreter<'a> {}
