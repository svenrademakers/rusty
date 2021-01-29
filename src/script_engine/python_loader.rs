use crate::script_interfaces;

struct PythonLoader {
    scripts : vec::Vec<Script>,
}

impl script_interfaces::ScriptFileLoader for PythonLoader {
    fn load(&self) -> bool {
        false
    }
    fn get_scripts(&self) -> vec::Vec<Script>
    {
        return vec::Vec<Script>::new();
    }
}
