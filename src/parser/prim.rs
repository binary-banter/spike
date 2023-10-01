use crate::parser::expression::parse_expression;
use crate::parser::operation::parse_operation;
use crate::parser::{trim0, trim1, Expression};
use nom::character::complete::char;
use nom::multi::many0;
use nom::sequence::{delimited, pair};
use nom::IResult;
use nom::Parser;

pub fn parse_prim(input: &str) -> IResult<&str, Expression> {
    delimited(
        char('('),
        pair(trim0(parse_operation), many0(trim1(parse_expression))).map(
            |(operation, arguments)| Expression::Prim {
                operation,
                arguments,
            },
        ),
        trim0(char(')')),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::prim::parse_prim;
    use crate::parser::{Expression, Operation};

    #[test]
    fn add() {
        assert_eq!(
            parse_prim("(+ 10 32)").unwrap().1,
            Expression::Prim {
                operation: Operation::Plus,
                arguments: vec![Expression::Int(10), Expression::Int(32)]
            }
        )
    }
}
