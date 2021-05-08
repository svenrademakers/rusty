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
    file: PathBuf,
}

impl<'a> PyInterpreter<'a> {
    pub fn new(file: &Path) -> Self {
        PyInterpreter {
            callables: SparseSecondaryMap::new(),
            _gil: Python::acquire_gil(),
            py: unsafe { Python::assume_gil_acquired() },
            file: file.to_path_buf(),
        }
    }

    // fn load_contents(
    //     &mut self,
    //     contents: &str,
    //     path: &str,
    // ) -> Result<(Vec<Script>, Vec<ParseError>), Box<dyn Error>> {
    //     let module_name = path.file_stem().and_then(OsStr::to_str).unwrap();
    //     let (objects, errors) = self.bind_to_python(&contents, &path, &module_name);
    //     let mut scripts = Vec::new();
    //     for (name, obj) in objects {
    //         let mut script = authority.request_script_entry(InterpreterType::Python);
    //         self.callables.insert(script.get_key(), obj);
    //         script.name = name;

    //         let arguments = PyInterpreter::get_arguments(obj);
    //         if !arguments.is_empty() {
    //             script.argument_type = arguments;
    //         }

    //         script.file = path.to_string_lossy().to_string();

    //         if obj.hasattr("__doc__")? {
    //             let doc = obj.getattr("__doc__").unwrap().to_string();
    //             if doc != "None" {
    //                 script.description = doc;
    //             }
    //         }

    //         scripts.push(script);
    //     }

    //     Ok((scripts, errors))
    // }

    // fn bind_to_python(
    //     &self,
    //     contents: &str,
    //     path: &Path,
    //     module_name: &str,
    // ) -> (Vec<(String, &'a PyAny)>, Vec<ParseError>) {
    //     let mut objects = Vec::new();
    //     let mut errors = Vec::new();
    //     let filename = path.to_string_lossy().to_string();
    //     let module = PyModule::from_code(self.py, &contents, &filename, &module_name);

    //     if let Err(e) = module {
    //         let error_tuple: ParseError = ParseError {
    //             filename: filename,
    //             message: e.pvalue(self.py).to_string(),
    //             traceback: e.ptraceback(self.py).to_object(self.py).to_string(),
    //         };
    //         errors.push(error_tuple);
    //     } else if let Ok(m) = module {
    //         for obj in m.dict().keys() {
    //             let name = obj.str().unwrap().to_str().unwrap().to_string();
    //             let obj = m.get(&name).unwrap();

    //             if obj.is_callable() {
    //                 objects.push((name, obj));
    //             }
    //         }
    //     }
    //     (objects, errors)
    // }

    // fn get_arguments(py_any: &PyAny) -> Vec<ArgumentType> {
    //     let mut args = Vec::new();
    //     for var_name in py_any.getattr("__code__").unwrap().getattr("co_varnames") {
    //         let argument_tuple = var_name.downcast::<PyTuple>().unwrap();
    //         if !argument_tuple.is_empty() {
    //             let name = argument_tuple.get_item(0).to_string();
    //             args.push(ArgumentType::String(name));
    //         }
    //     }

    //     args
    // }
}

impl<'a> Interpreter for PyInterpreter<'a> {
    fn parse(&mut self) -> Result<usize, InterpreterError> {
        Err(InterpreterError::Error)
    }
    fn load(&mut self, keys: &[ScriptKey]) -> Result<ScriptInterpreterResult, InterpreterError> {
        Err(InterpreterError::Error)
    }
    fn call(&self, key: ScriptKey, args: &[Box<dyn Any>]) -> Result<(), CallError> {
        Err(CallError::KeyNotPresent)
    }
}

// impl<'a> Interpreter for PyInterpreter<'a> {
//     fn parse(&mut self, file: &Path) -> Result<ScriptInterpreterResult, InterpreterError> {
//         let contents = std::fs::read_to_string(path)?;
//         self.load_contents(&contents, &path, authority)
//     }

//     fn update_keys(switch: &[(u64, ScriptKey)]) {}
// }

// impl<'a> CallContext for PyInterpreter<'a> {
//     fn call(&self, key: ScriptKey, args: &[Box<dyn Any>]) -> Result<(), CallError> {
//         if let Some(x) = self.callables.get(script_key) {
//             if args.len() == 0 {
//                 x.call0()?;
//             } else {
//                 let mut py_arguments = Vec::new();
//                 for arg in args {
//                     let typ = (&*arg).type_id();
//                     if typ == TypeId::of::<String>() {
//                         py_arguments.push(arg.downcast_ref::<String>().unwrap());
//                     } else {
//                         warn!("cannot convert one of the arguments");
//                     }
//                 }
//                 let py_args = PyTuple::new(self.py, py_arguments);
//                 x.call1(py_args)?;
//             }
//         }
//         Ok(true)
//     }
// }
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parse_error_added() {
//         let mut script_store = ScriptStore::new();
//         let mut py_interpreter = PyInterpreter::new();
//         py_interpreter
//             .load_contents(
//                 "adsfasdf",
//                 &std::path::PathBuf::from("/my/path/sven.py"),
//                 &mut script_store,
//             )
//             .unwrap();
//         assert_eq!(script_store.parse_errors[0].filename, "/my/path/sven.py");
//         assert!(!script_store.parse_errors[0].message.is_empty());
//     }
//     #[test]
//     fn parse_py_files() {
//         let mut script_store = ScriptStore::new();
//         let mut py_interpreter = PyInterpreter::new();
//         py_interpreter
//             .load_contents(
//                 concat!(
//                     "def test_123(wat):\n\t\"\"\"this is a test",
//                     " doc\"\"\"\n\tprint(\"hoi\")\n",
//                     "def test_2():\n\tprint(\"test2\")\n"
//                 ),
//                 &std::path::PathBuf::from("/my/path/sven.py"),
//                 &mut script_store,
//             )
//             .unwrap();
//         let mut iter = script_store.scripts.iter();
//         let (key, typ) = iter.next().unwrap();
//         assert_eq!(typ, &InterpreterType::Python);
//         assert_eq!(script_store.names[key], "test_123");
//         assert_eq!(script_store.files[key], "/my/path/sven.py");
//         assert_eq!(script_store.description[key], "this is a test doc");
//         assert_eq!(
//             script_store.argument_type[key][0],
//             ArgumentType::String("wat".to_string())
//         );

//         let (key, typ) = iter.next().unwrap();
//         assert_eq!(typ, &InterpreterType::Python);
//         assert_eq!(script_store.names[key], "test_2");
//         assert_eq!(script_store.files[key], "/my/path/sven.py");
//         assert_eq!(script_store.description.get(key), None);
//         assert_eq!(script_store.argument_type.get(key), None);

//         assert_eq!(script_store.parse_errors.len(), 0);
//     }
// }

unsafe impl<'a> Send for PyInterpreter<'a> {}
unsafe impl<'a> Sync for PyInterpreter<'a> {}
