use std::boxed::Box;
use std::vec::Vec;

enum ScriptArguments {
  Boolean,
  Int,
  Uint,
  Float,
  Text,
  List(Box<Vec<ScriptArguments>>),
}

#[derive(PartialEq, Clone, Hash, Debug)]
struct Script {
  name: String,
  description: String,
  handle: i32,
  arguments: Vec<ScriptArguments>,
  argument_descriptions: Vec<String>,
}

pub trait ScriptLoader {
  scripts: Vec<Box<Script>>;
  fn parse(&self) -> bool;
  fn get_scripts(&self) -> &[Box<Script>] {
    self.scripts
  }
}