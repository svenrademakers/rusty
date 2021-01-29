use crate::script_interfaces;

mod script_engine {
struct PythonLoader {
    scripts : vec::Vec<Script>,
}

impl script_interfaces::ScriptFileLoader for PythonLoader {
    fn load(&self) -> bool {
        scripts.insert(Script {
          name : "test_script".to_string(),
          description : "This is a wonderful test script".to_string(),
          arguments: vec::Vec::new(),
          argument_descriptions: vec::Vec::new(),
          }
        false
    }

    fn get_scripts(&self) -> &[_]
    {
        scripts
    }
}
}