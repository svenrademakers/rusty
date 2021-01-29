mod script_engine
{

Enum ScriptArguments {
  Boolean,
  Int,
  Uint,
  Float,
  Text,
  List(vec::Vec<ScriptArguments>)
}

#[derive(PartialEq, Clone, Copy, Hash, Debug)]
struct Script {
  name : String,
  description: String,
  handle : i32,
  arguments : vec::Vec<ScriptArguments>,
  argument_descriptions : vec::Vec<String>,
}

pub trait GetScripts {
  fn load(&self) -> bool;
  fn get_scripts(&self) -> &[_];
}

pub struct ScriptLoader<T : GetScripts> {
  pub scripts : vec::Vec<box::Box<Script>>,
}

impl<T> ScriptLoader<T> {

}
}