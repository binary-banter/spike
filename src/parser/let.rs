use crate::parser;
use crate::parser::{expression, identifier, Expression};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::sequence::{delimited, pair, tuple};
use nom::{IResult, Parser};

pub fn parse_let(input: &str) -> IResult<&str, Expression<'_>> {
    delimited(
        char('('),
        tuple((
            parser::trim0(tag("let")),
            parser::trim1(delimited(
                char('('),
                pair(
                    parser::trim0(identifier::parse_identifier),
                    parser::trim1(expression::parse_expression),
                ),
                parser::trim0(char(')')),
            )),
            parser::trim1(expression::parse_expression),
        )),
        parser::trim0(char(')')),
    )
    .map(|(_, (name, binding), body)| Expression::Let {
        name,
        binding: Box::new(binding),
        body: Box::new(body),
    })
    .parse(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::r#let::parse_let;
    use crate::parser::Expression;

    #[test]
    fn simple() {
        assert_eq!(
            parse_let("(let (x 42) x)").unwrap().1,
            Expression::Let {
                name: "x",
                binding: Box::new(Expression::Int(42)),
                body: Box::new(Expression::Var { name: "x" })
            }
        )
    }
}
