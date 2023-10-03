use crate::language::lvar::Expr;
use crate::parser::expression::parse_expression;
use crate::parser::operation::parse_operation;
use crate::parser::{trim0, trim1};
use nom::character::complete::char;
use nom::multi::many0;
use nom::sequence::{delimited, pair};
use nom::IResult;
use nom::Parser;

pub fn parse_prim(input: &str) -> IResult<&str, Expr> {
    delimited(
        char('('),
        pair(trim0(parse_operation), many0(trim1(parse_expression)))
            .map(|(op, args)| Expr::Prim { op, args }),
        trim0(char(')')),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::language::lvar::{Expr, Op};
    use crate::parser::prim::parse_prim;

    #[test]
    fn add() {
        assert_eq!(
            parse_prim("(+ 10 32)").unwrap().1,
            Expr::Prim {
                op: Op::Plus,
                args: vec![Expr::Int { val: 10 }, Expr::Int { val: 32 }]
            }
        )
    }
}
