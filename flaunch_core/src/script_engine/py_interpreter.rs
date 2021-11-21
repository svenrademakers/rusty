extern crate pyo3;

use std::any::TypeId;
use std::collections::HashMap;

use crate::script_engine::interpreter::*;
use crate::script_engine::*;

use log::info;

use pyo3::types::*;
pub use pyo3::{
    prelude::*,
    types::{PyModule, PyTuple},
};

#[derive(Debug)]
pub struct PyInterpreter {
    annotation_mod: Py<PyModule>,
}

impl Default for PyInterpreter {
    fn default() -> Self {
        let gil = Python::acquire_gil();
        let py = gil.python();
        info!("python version = {}", py.version());
        let module = PyModule::from_code(
            py,
            &include_str!("py_annotation.py"),
            "py_annotation.py",
            "py_annotation",
        )
        .unwrap();
        PyInterpreter {
            annotation_mod: module.into_py(gil.python()),
        }
    }
}

impl Interpreter for PyInterpreter {
    fn parse(&self, content: &[u8], file: &Path) -> ParseResult {
        let mut scripts = Vec::new();
        let mut callables: Vec<(u64, Arc<dyn Callable>)> = Vec::new();
        let mut errors = Vec::new();

        let as_str = std::str::from_utf8(content).unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();

        let globals = pyo3::types::PyDict::new(py);
        if let Err(e) = Python::run(py, as_str, None, Some(globals)) {
            let error_tuple: ParseError = ParseError {
                filename: file.to_string_lossy().to_string(),
                message: e.pvalue(py).to_string(),
                traceback: e.ptraceback(py).to_object(py).to_string(),
            };
            errors.push(error_tuple);
            info!("err {:?}", errors);
        } else {
            if let Some(func_call) = globals.get_item("flaunch_callables") {
                let mut py_call = PyCallable::default();
                for (key, value) in func_call.downcast::<PyDict>().unwrap() {
                    let descriptions = value.downcast::<PyDict>().unwrap();
                    if let Ok(func) = key.downcast::<PyFunction>() {
                        if let Ok(name) = func.getattr("__name__") {
                            let script = create_script_object(name, file, func, descriptions);
                            py_call.insert(script.get_key().unwrap(), func.to_object(py));
                            scripts.push(script);
                        }
                    }
                }
                let keys = py_call.keys();
                let rc: Arc<dyn Callable> = Arc::new(py_call);
                callables = keys.iter().map(|key| (key.clone(), rc.clone())).collect();
            }
        }

        (scripts, callables, errors)
    }
}

fn create_script_object(
    name: &PyAny,
    file: &Path,
    func: &PyFunction,
    descriptions: &PyDict,
) -> Script {
    let mut script = Script::new(name.to_string(), InterpreterType::Python);
    script.file = file.to_path_buf();
    let annotations = func
        .getattr("__annotations__")
        .unwrap()
        .downcast::<PyDict>()
        .unwrap();
    for (key, value) in annotations.iter() {
        let mut description = String::new();
        if let Some(des) = descriptions.get_item(key) {
            description = des.to_string().trim().to_string();
        }
        script
            .arguments
            .push((key.to_string(), get_flaunch_type(value), description));
    }
    if func.hasattr("__doc__").unwrap() {
        let doc = func.getattr("__doc__").unwrap().to_string();
        if doc != "None" {
            script.description = doc.trim().to_string();
        }
    }
    script
}

fn get_flaunch_type(any: &PyAny) -> ArgumentType {
    if let Ok(typ) = any.downcast::<PyType>() {
        return match typ.name() {
            Ok("str") => ArgumentType::String("".to_string()),
            Ok("int") => ArgumentType::Int(0),
            _ => ArgumentType::NotSpecified,
        };
    }
    ArgumentType::NotSpecified
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
                "@flaunch(wat=\"Print Statement\", number=\"Given Number\")",
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
