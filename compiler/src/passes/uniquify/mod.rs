pub mod uniquify;

use std::collections::HashMap;
use crate::passes::parse::Def;
use crate::passes::type_check::TExpr;
use crate::utils::gen_sym::UniqueSym;

#[derive(Debug, PartialEq)]
pub struct PrgUniquified<'p> {
    pub defs: HashMap<UniqueSym<'p>, Def<'p, UniqueSym<'p>, TExpr<'p, UniqueSym<'p>>>>,
    pub entry: UniqueSym<'p>,
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn unique([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let uniquified_program = program.type_check().unwrap().uniquify();
        let mut io = TestIO::new(input);
        let result = uniquified_program.interpret(&mut io);

        assert_eq!(result, expected_return.into(), "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as uniquify => unique }
}
