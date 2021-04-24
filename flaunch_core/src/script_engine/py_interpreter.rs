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
        parse_errors: &mut Vec<ParseError>,
    ) -> Vec<(String, &'a PyAny)> {
        let mut objects = Vec::new();
        let module = PyModule::from_code(self.py, &contents, &filename, &module_name);

        if let Err(e) = module {
            let error_tuple: ParseError = ParseError {
                filename: format!("{}/{}", module_name.to_string(), filename.to_string()),
                message: e.pvalue(self.py).to_string(),
                traceback: e.ptraceback(self.py).to_object(self.py).to_string(),
            };
            parse_errors.push(error_tuple);
        } else if let Ok(m) = module {
            for obj in m.dict().keys() {
                let name = obj.str().unwrap().to_str().unwrap().to_string();
                let obj = m.get(&name).unwrap();

                if obj.is_callable() {
                    objects.push((name, obj));
                }
            }
        }
        objects
    }

    pub fn get_arguments(py_any: &PyAny) -> Vec<Argument> {
        let mut args = Vec::new();
        for var_name in py_any.getattr("__code__").unwrap().getattr("co_varnames") {
            let argument_tuple = var_name.downcast::<PyTuple>().unwrap();
            if !argument_tuple.is_empty() {
                let name = argument_tuple.get_item(0).to_string();
                args.push(Argument::String(name));
            }
        }

        args
    }

    fn load_contents(
        &mut self,
        contents: &str,
        path: &Path,
        script_store: &mut ScriptStore,
    ) -> Result<(), Box<dyn Error>> {
        let filename = path.file_name().and_then(OsStr::to_str).unwrap();
        let module_name = path.file_stem().and_then(OsStr::to_str).unwrap();

        for (name, obj) in self.bind_to_python(
            &contents,
            &filename,
            &module_name,
            &mut script_store.parse_errors,
        ) {
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

        Ok(())
    }
}

impl<'a> Interpreter for PyInterpreter<'a> {
    fn parse(&mut self, path: &Path, script_store: &mut ScriptStore) -> Result<(), Box<dyn Error>> {
        if let Ok(contents) = std::fs::read_to_string(path) {
            self.load_contents(&contents, &path, script_store)?;
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

unsafe impl<'a> Send for PyInterpreter<'a> {}
unsafe impl<'a> Sync for PyInterpreter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_added() {
        let mut script_store = ScriptStore::new();
        let mut py_interpreter = PyInterpreter::new();
        py_interpreter
            .load_contents(
                "adsfasdf",
                &std::path::PathBuf::from("/my/path/sven.py"),
                &mut script_store,
            )
            .unwrap();
        assert_eq!(script_store.parse_errors[0].filename, "/my/path/sven.py");
        assert!(!script_store.parse_errors[0].message.is_empty());
    }
    #[test]
    fn parse_py_files() {
        let mut script_store = ScriptStore::new();
        let mut py_interpreter = PyInterpreter::new();
        py_interpreter
            .load_contents(
                "def test_123(wat):\n\tprint(\"hoi\")\n",
                &std::path::PathBuf::from("/my/path/sven.py"),
                &mut script_store,
            )
            .unwrap();

        let key = script_store.names.iter().next().unwrap().0;
        assert_eq!(script_store.names[key], "test_123");
        assert_eq!(script_store.files[key], "/my/path/sven.py");
    }
}
