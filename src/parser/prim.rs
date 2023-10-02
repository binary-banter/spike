use crate::parser::expression::parse_expression;
use crate::parser::operation::parse_operation;
use crate::parser::{trim0, trim1, Expr};
use nom::character::complete::char;
use nom::multi::many0;
use nom::sequence::{delimited, pair};
use nom::IResult;
use nom::Parser;

pub fn parse_prim(input: &str) -> IResult<&str, Expr> {
    delimited(
        char('('),
        pair(trim0(parse_operation), many0(trim1(parse_expression))).map(
            |(operation, arguments)| Expr::Prim {
                op: operation,
                args: arguments,
            },
        ),
        trim0(char(')')),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::prim::parse_prim;
    use crate::parser::{Expr, Operation};

    #[test]
    fn add() {
        assert_eq!(
            parse_prim("(+ 10 32)").unwrap().1,
            Expr::Prim {
                op: Operation::Plus,
                args: vec![Expr::Int(10), Expr::Int(32)]
            }
        )
    }
}
