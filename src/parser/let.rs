use crate::language::lvar::Expr;
use crate::parser::expression::parse_expression;
use crate::parser::identifier::parse_identifier;
use crate::parser::{trim0, trim1};
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
                pair(trim0(parse_identifier), trim1(parse_expression)),
                trim0(char(')')),
            )),
            trim1(parse_expression),
        )),
        trim0(char(')')),
    )
    .map(|(_, (sym, bnd), bdy)| Expr::Let {
        sym,
        bnd: Box::new(bnd),
        bdy: Box::new(bdy),
    })
    .parse(input)
}

#[cfg(test)]
mod tests {
    use crate::language::lvar::Expr;
    use crate::parser::r#let::parse_let;

    #[test]
    fn simple() {
        assert_eq!(
            parse_let("(let (x 42) x)").unwrap().1,
            Expr::Let {
                sym: "x",
                bnd: Box::new(Expr::Int { val: 42 }),
                bdy: Box::new(Expr::Var {
                    sym: "x"
                })
            }
        )
    }
}
