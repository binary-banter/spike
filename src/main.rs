use crate::parser::parse_program;

mod parser;

fn main() {
    parse_program("(let (x 42) x)").unwrap();
}
