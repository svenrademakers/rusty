extern crate pyo3;

use std::collections::HashMap;
use std::{any::TypeId, ffi::OsStr};

use crate::script_engine::interpreter::*;
use crate::script_engine::*;

pub use pyo3::{
    prelude::*,
    types::{PyModule, PyTuple},
};

#[derive(Debug, Default)]
pub struct PyInterpreter {}

impl Interpreter for PyInterpreter {
    fn parse(&self, content: &[u8], file: &Path) -> ParseResult {
        let mut scripts = Vec::new();
        let mut callables: Vec<(u64, Arc<dyn Callable>)> = Vec::new();
        let mut errors = Vec::new();

        let filename = file.to_string_lossy().to_string();
        let module_name: &str = file.file_stem().and_then(OsStr::to_str).unwrap();
        let as_str = std::str::from_utf8(content).unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();
        let module = PyModule::from_code(py, &as_str, &filename, &module_name);
        if let Err(e) = module {
            let error_tuple: ParseError = ParseError {
                filename: filename,
                message: e.pvalue(py).to_string(),
                traceback: e.ptraceback(py).to_object(py).to_string(),
            };
            errors.push(error_tuple);
        } else if let Ok(m) = module {
            let mut py_call = PyCallable::default();

            for obj in m.dict().keys() {
                let name = obj.str().unwrap().to_str().unwrap().to_string();
                let obj = m.get(&name).unwrap();

                if obj.is_callable() {
                    let script = create_script_obj(obj, file, name);
                    let key = script.get_key().unwrap();
                    py_call.insert(key, obj.into_py(py));
                    scripts.push(script);
                }
            }
            let keys = py_call.keys();
            let rc: Arc<dyn Callable> = Arc::new(py_call);
            callables = keys.iter().map(|key| (key.clone(), rc.clone())).collect();
        }

        (scripts, callables, errors)
    }
}

#[derive(Default, Debug)]
pub struct PyCallable {
    callables: HashMap<u64, PyObject>,
}

impl PyCallable {
    pub fn insert(&mut self, key: u64, value: PyObject) {
        self.callables.insert(key, value);
    }

    pub fn keys(&self) -> Vec<u64> {
        self.callables.keys().cloned().collect()
    }
}
impl Callable for PyCallable {
    fn call(&self, key: u64, args: &[Box<dyn Any>]) -> Result<(), CallError> {
        if let Some(obj) = self.callables.get(&key) {
            let gil = Python::acquire_gil();
            let py = gil.python();

            if args.len() == 0 {
                obj.call0(py).unwrap();
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
                obj.call1(py, py_args).unwrap();
            }
            Ok(())
        } else {
            return Err(CallError::KeyNotPresent(key));
        }
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

fn create_script_obj(obj: &PyAny, file: &Path, name: String) -> Script {
    let mut script = Script::new(name, InterpreterType::Python);
    let arguments = get_arguments(&obj);
    if !arguments.is_empty() {
        script.argument_type = arguments;
    }
    script.file = file.to_path_buf();
    if obj.hasattr("__doc__").unwrap() {
        let doc = obj.getattr("__doc__").unwrap().to_string();
        if doc != "None" {
            script.description = doc;
        }
    }
    script
}

unsafe impl Send for PyInterpreter {}
unsafe impl Sync for PyInterpreter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_added() {
        let py_interpreter = PyInterpreter::default();
        let result = py_interpreter.parse(
            "adsfasdf".as_bytes(),
            &std::path::PathBuf::from("/my/path/sven.py"),
        );

        assert_eq!(result.0.len(), 0);
        assert_eq!(result.1.len(), 0);
        assert_eq!(result.2.len(), 1);
    }

    #[test]
    fn parse_py_files() {
        let py_interpreter = PyInterpreter::default();
        let (scripts, callables, errors) = py_interpreter.parse(
            concat!(
                "def test_123(wat):\n\t\"\"\"this is a test",
                " doc\"\"\"\n\tprint(\"hoi\")\n",
                "def test_2():\n\tprint(\"test2\")\n"
            )
            .as_bytes(),
            &std::path::PathBuf::from("/my/path/sven.py"),
        );
        assert_eq!(errors.len(), 0);
        assert_eq!(scripts[0].name, "test_123".to_string());
        assert_eq!(scripts[0].file, PathBuf::from("/my/path/sven.py"));
        assert_eq!(scripts[0].description, "this is a test doc".to_string());
        assert_eq!(
            scripts[0].argument_type[0],
            ArgumentType::String("wat".to_string())
        );

        assert_eq!(scripts[1].name, "test_2".to_string());
        assert_eq!(scripts[1].file, PathBuf::from("/my/path/sven.py"));
        assert!(scripts[1].description.is_empty());
        assert!(scripts[1].argument_type.is_empty());

        assert_eq!(callables.len(), 2);
        assert!(callables
            .iter()
            .find(|x| { x.0 == scripts[0].get_key().unwrap() })
            .is_some());
        assert!(callables
            .iter()
            .find(|x| { x.0 == scripts[1].get_key().unwrap() })
            .is_some());
    }

    #[test]
    fn keys_are_the_same() {
        let py_interpreter = PyInterpreter::default();
        let (scripts, _callables, _errors) = py_interpreter.parse(
            concat!(
                "def test_123(wat):\n\t\"\"\"this is a test",
                " doc\"\"\"\n\tprint(\"hoi\")\n",
            )
            .as_bytes(),
            &std::path::PathBuf::from("/my/path/sven.py"),
        );
        assert_eq!(scripts[0].get_key(), scripts[0].get_key());
    }
}
