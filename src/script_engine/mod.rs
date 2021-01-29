pub mod python_loader;
pub mod script_interfaces;

pub struct ScriptManager {
    parsers: std::collections::HashMap,
}

impl ScriptManager {
    pub fn new() -> Self {
        ScriptManager {}
    }

    fn load() -> bool {
        false
    }
}
