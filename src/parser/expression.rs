use crate::language::lvar::Expr;
use crate::parser::int::parse_int;
use crate::parser::prim::parse_prim;
use crate::parser::r#let::parse_let;
use crate::parser::var::parse_var;
use nom::branch::alt;
use nom::IResult;

pub fn parse_expression<'p>(input: &'p str) -> IResult<&'p str, Expr<&'p str>> {
    alt((parse_int, parse_var, parse_prim, parse_let))(input)
}

#[cfg(test)]
mod tests {
    use crate::language::lvar::Expr;
    use crate::parser::expression::parse_expression;

    #[test]
    fn int() {
        assert_eq!(parse_expression("42").unwrap().1, Expr::Int { val: 42 })
    }
}
