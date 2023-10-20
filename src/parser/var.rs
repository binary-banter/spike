use nom::error::ErrorKind;
use crate::language::lvar::Expr;
use crate::parser::identifier::parse_identifier;
use nom::IResult;

const RESERVED_KEYWORDS: [&str; 7] = ["t", "f", "fn", "if", "let", "Int", "Bool"];

pub fn parse_var(input: &str) -> IResult<&str, Expr<&str>> {
    let (rest, sym) = parse_identifier(input)?;

    if RESERVED_KEYWORDS.contains(&sym) {
        return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Verify)));
    }

    return Ok((rest, Expr::Var { sym }))
}
