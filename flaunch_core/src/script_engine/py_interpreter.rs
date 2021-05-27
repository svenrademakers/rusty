extern crate pyo3;

use std::{any::TypeId, ffi::OsStr};

use crate::script_engine::interpreter::*;
use crate::script_engine::*;

pub use pyo3::{
    prelude::*,
    types::{PyModule, PyTuple},
};

pub struct PyInterpreter {
    callables: SparseSecondaryMap<ScriptKey, PyObject>,
    pending: Vec<(String, PyObject)>,
    file: PathBuf,
}

impl PyInterpreter {
    pub fn new() -> Self {
        PyInterpreter {
            callables: SparseSecondaryMap::new(),
            pending: Vec::new(),
            file: PathBuf::new(),
        }
    }

    fn get_arguments(py_any: &PyAny) -> Vec<ArgumentType> {
        let mut args = Vec::new();
        for var_name in py_any.getattr("__code__").unwrap().getattr("co_varnames") {
            let argument_tuple = var_name.downcast::<PyTuple>().unwrap();
            if !argument_tuple.is_empty() {
                let name = argument_tuple.get_item(0).to_string();
                args.push(ArgumentType::String(name));
            }
        }

        args
    }
}

impl Interpreter for PyInterpreter {
    fn parse(
        &mut self,
        content: &[u8],
        file: &Path,
    ) -> Result<(usize, Vec<ParseError>), InterpreterError> {
        let filename = file.to_string_lossy().to_string();
        let module_name: &str = file.file_stem().and_then(OsStr::to_str).unwrap();
        let as_str = std::str::from_utf8(content).unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();
        let module = PyModule::from_code(py, &as_str, &filename, &module_name);
        let mut errors = Vec::new();
        self.file = file.to_path_buf();
        if let Err(e) = module {
            let error_tuple: ParseError = ParseError {
                filename: filename,
                message: e.pvalue(py).to_string(),
                traceback: e.ptraceback(py).to_object(py).to_string(),
            };
            errors.push(error_tuple);
        } else if let Ok(m) = module {
            for obj in m.dict().keys() {
                let name = obj.str().unwrap().to_str().unwrap().to_string();
                let obj = m.get(&name).unwrap();

                if obj.is_callable() {
                    self.pending.push((name, obj.to_object(py)));
                }
            }
        }

        Ok((self.pending.len(), errors))
    }

    fn load(&mut self, mut keys: Vec<ScriptKey>) -> Result<Vec<Script>, InterpreterError> {
        if keys.len() < self.pending.len() {
            return Err(InterpreterError::NotEnoughKeys {
                needed: self.pending.len(),
                provided: keys.len(),
            });
        }

        let mut scripts = Vec::new();
        let gil = Python::acquire_gil();
        let py = gil.python();

        while !self.pending.is_empty() {
            let (name, obj) = self.pending.pop().unwrap();
            let mut script = Script::new(keys.pop().unwrap());

            script.name = name;

            let arguments = PyInterpreter::get_arguments(obj.as_ref(py));
            if !arguments.is_empty() {
                script.argument_type = arguments;
            }

            script.file = self.file.clone();

            if obj.as_ref(py).hasattr("__doc__").unwrap() {
                let doc = obj.getattr(py, "__doc__").unwrap().to_string();
                if doc != "None" {
                    script.description = doc;
                }
            }

            self.callables.insert(script.key, obj);

            scripts.push(script);
        }
        Ok(scripts)
    }

    fn call(&self, key: ScriptKey, args: &[Box<dyn Any>]) -> Result<(), CallError> {
        match self.callables.get(key) {
            Some(x) => {
                let gil = Python::acquire_gil();
                let py = gil.python();

                if args.len() == 0 {
                    x.call0(py).unwrap();
                } else {
                    let mut py_arguments = Vec::new();
                    for arg in args {
                        let typ = (&*arg).type_id();
                        if typ == TypeId::of::<String>() {
                            py_arguments.push(arg.downcast_ref::<String>().unwrap());
                        } else {
                            return Err(CallError::WrongArguments);
                        }
                    }
                    let py_args = PyTuple::new(py, py_arguments);
                    x.call1(py, py_args).unwrap();
                }
                Ok(())
            }
            None => Err(CallError::KeyNotPresent(key)),
        }
    }
}

unsafe impl Send for PyInterpreter {}
unsafe impl Sync for PyInterpreter {}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::*;

    #[test]
    fn parse_error_added() {
        let mut py_interpreter = PyInterpreter::new();
        let result = py_interpreter
            .parse(
                "adsfasdf".as_bytes(),
                &std::path::PathBuf::from("/my/path/sven.py"),
            )
            .unwrap();

        assert_eq!(result.0, 0);
        assert_eq!(result.1.len(), 1);
    }

    #[test]
    fn parse_py_files() {
        let mut keys: SlotMap<ScriptKey, InterpreterArc> = SlotMap::default();
        let arc = create_interpreter_for_file(&PathBuf::from("w.py")).unwrap();
        let mut py_interpreter = block_on(arc.lock());

        let result = py_interpreter
            .parse(
                concat!(
                    "def test_123(wat):\n\t\"\"\"this is a test",
                    " doc\"\"\"\n\tprint(\"hoi\")\n",
                    "def test_2():\n\tprint(\"test2\")\n"
                )
                .as_bytes(),
                &std::path::PathBuf::from("/my/path/sven.py"),
            )
            .unwrap();
        assert_eq!(result.0, 2);
        assert_eq!(result.1.len(), 0);

        assert_eq!(
            py_interpreter.load(Vec::new()),
            Err(InterpreterError::NotEnoughKeys {
                needed: 2,
                provided: 0
            }),
        );

        let mut res = Vec::new();
        for _n in 0..result.0 {
            res.push(keys.insert(arc.clone()));
        }

        let result = py_interpreter.load(res).unwrap();

        assert_eq!(result[1].name, "test_123".to_string());
        assert_eq!(result[1].file, PathBuf::from("/my/path/sven.py"));
        assert_eq!(result[1].description, "this is a test doc".to_string());
        assert_eq!(
            result[1].argument_type[0],
            ArgumentType::String("wat".to_string())
        );

        assert_eq!(result[0].name, "test_2".to_string());
        assert_eq!(result[0].file, PathBuf::from("/my/path/sven.py"));
        assert!(result[0].description.is_empty());
        assert!(result[0].argument_type.is_empty());
    }
}
