use std::boxed::Box;
use std::vec::Vec;

enum Argument {
  Boolean,
  Int,
  Uint,
  Float,
  Text,
  List(Box<Vec<Argument>>),
}

#[derive(PartialEq, Clone, Hash, Debug)]
struct Script {
  name: String,
  description: String,
  handle: i32,
  arguments: Vec<Argument>,
  argument_descriptions: Vec<String>,
}
