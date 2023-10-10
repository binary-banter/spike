pub mod lvar;
pub mod x86var;
pub mod value;

use std::io::stdin;
use std::vec::IntoIter;
use crate::interpreter::value::Value;

pub trait IO {
    fn read(&mut self) -> Value;
    fn print(&mut self, v: Value);
}

struct StdIO {}

impl IO for StdIO {
    fn read(&mut self) -> Value {
        print!("> ");
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("IO error or something");
        input
            .trim_end()
            .parse()
            .expect("Provided input was not a valid i64")
    }

    fn print(&mut self, v: Value) {
        println!("{v}");
    }
}

pub struct TestIO {
    inputs: IntoIter<Value>,
    outputs: Vec<Value>,
}

impl TestIO {
    pub fn new(inputs: Vec<Value>) -> Self {
        Self {
            inputs: inputs.into_iter(),
            outputs: Vec::new(),
        }
    }

    pub fn outputs(&self) -> &Vec<Value> {
        &self.outputs
    }
}

impl IO for TestIO {
    fn read(&mut self) -> Value {
        self.inputs
            .next()
            .expect("Test tried to read more input than were available.")
    }

    fn print(&mut self, v: Value) {
        self.outputs.push(v);
    }
}
