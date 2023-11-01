pub mod check;
pub mod error;
mod uncover_globals;
mod util;
mod validate_expr;
mod validate_prim;
mod validate_struct;
mod validate_type;

use crate::passes::parse::{Def, Expr};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

pub type PrgTypeChecked<'p> = PrgGenericVar<'p, &'p str>;

#[cfg(test)]
mod tests {
    use crate::passes::parse::parse::parse_program;
    use test_each_file::test_each_file;

    fn check([test]: [&str; 1], should_fail: bool) {
        let mut test = test.split('#');
        let program = test.nth(3).unwrap().trim();
        let program = parse_program(program).unwrap();
        let res = program.type_check();

        match (res, should_fail) {
            (Ok(_), true) => panic!("Program should not pass type-checking."),
            (Err(e), false) => {
                panic!("Program should have passed type-checking, but returned error: '{e}'.")
            }
            _ => {}
        }
    }

    test_each_file! { for ["test"] in "./programs/good" as type_check_succeed => |p| check(p, false) }
    test_each_file! { for ["test"] in "./programs/fail/type_check" as type_check_fail => |p| check(p, true) }
}

/// A generic program with global definitions and an entry point.
#[derive(Debug, PartialEq)]
pub struct PrgGenericVar<'p, A: Copy + Hash + Eq + Display> {
    /// The global program definitions.
    pub defs: HashMap<A, Def<'p, A, Expr<'p, A>>>,
    /// The symbol representing the entry point of the program.
    pub entry: A,
}
