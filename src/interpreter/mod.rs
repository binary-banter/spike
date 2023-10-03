pub mod lvar;
mod x86var;

use std::io::stdin;
use std::vec::IntoIter;

pub trait IO {
    fn read(&mut self) -> i64;
    fn print(&mut self, v: i64);
}

struct StdIO {}

impl IO for StdIO {
    fn read(&mut self) -> i64 {
        print!("> ");
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("IO error or something");
        input.parse().expect("Provided input was not a valid i64")
    }

    fn print(&mut self, v: i64) {
        println!("{v}");
    }
}

pub struct TestIO {
    inputs: IntoIter<i64>,
    outputs: Vec<i64>,
}

impl TestIO {
    pub fn new(inputs: Vec<i64>) -> Self {
        Self {
            inputs: inputs.into_iter(),
            outputs: Vec::new(),
        }
    }

    pub fn outputs(&self) -> &Vec<i64> {
        &self.outputs
    }
}

impl IO for TestIO {
    fn read(&mut self) -> i64 {
        self.inputs
            .next()
            .expect("Test tried to read more input than were available.")
    }

    fn print(&mut self, v: i64) {
        self.outputs.push(v);
    }
}
