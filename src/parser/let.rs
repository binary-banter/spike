use crate::parser::{expression, identifier, trim0, trim1, Expr};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::sequence::{delimited, pair, tuple};
use nom::{IResult, Parser};

pub fn parse_let(input: &str) -> IResult<&str, Expr> {
    delimited(
        char('('),
        tuple((
            trim0(tag("let")),
            trim1(delimited(
                char('('),
                pair(
                    trim0(identifier::parse_identifier),
                    trim1(expression::parse_expression),
                ),
                trim0(char(')')),
            )),
            trim1(expression::parse_expression),
        )),
        trim0(char(')')),
    )
    .map(|(_, (name, binding), body)| Expr::Let {
        sym: name.to_string(),
        bnd: Box::new(binding),
        bdy: Box::new(body),
    })
    .parse(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::r#let::parse_let;
    use crate::parser::Expr;

    #[test]
    fn simple() {
        assert_eq!(
            parse_let("(let (x 42) x)").unwrap().1,
            Expr::Let {
                sym: "x".to_string(),
                bnd: Box::new(Expr::Int(42)),
                bdy: Box::new(Expr::Var {
                    sym: "x".to_string()
                })
            }
        )
    }
}
