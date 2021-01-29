mod script_engine::script_interfaces;

struct PythonLoader {}

impl script_interfaces::ScriptLoader for PythonLoader {}

impl ScriptLoader for PythonLoader {
    fn parse(&self) -> bool {
        scripts.insert(Script {
            name: "test_script".to_string(),
            description: "This is a wonderful test script".to_string(),
            arguments: vec::Vec::new(),
            argument_descriptions: vec::Vec::new(),
        });
        true
    }
}
