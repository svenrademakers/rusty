mod script_engine
{

Enum ScriptArguments {
  Boolean,
  Integer,
  Float,
  Text,
  List(vec::Vec<ScriptArguments>)
}

Struct Script {
  name : String,
  description: String,
  handle : i32,
  arguments : vec::Vec<ScriptArguments>,
  argument_descriptions : vec::Vec<String>,
}

pub trait ScriptFileLoader {
  fn load(&self) -> bool;
  fn get_scripts(&self) -> vec::Vec<Script>;
}

}