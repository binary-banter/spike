pub mod cvar;
pub mod lvar;
pub mod value;
pub mod x86var;

use crate::interpreter::value::Val;
use std::io::stdin;
use std::vec::IntoIter;

pub trait IO {
    fn read(&mut self) -> Val;
    fn print(&mut self, v: Val);
}

struct StdIO {}

impl IO for StdIO {
    fn read(&mut self) -> Val {
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

    fn print(&mut self, v: Val) {
        println!("{v}");
    }
}

pub struct TestIO {
    inputs: IntoIter<Val>,
    outputs: Vec<Val>,
}

impl TestIO {
    pub fn new(inputs: Vec<Val>) -> Self {
        Self {
            inputs: inputs.into_iter(),
            outputs: Vec::new(),
        }
    }

    pub fn outputs(&self) -> &Vec<Val> {
        &self.outputs
    }
}

impl IO for TestIO {
    fn read(&mut self) -> Val {
        self.inputs
            .next()
            .expect("Test tried to read more input than were available.")
    }

    fn print(&mut self, v: Val) {
        self.outputs.push(v);
    }
}
