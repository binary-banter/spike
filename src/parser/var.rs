use crate::language::lvar::Expr;
use crate::parser::identifier::parse_identifier;
use nom::IResult;

pub fn parse_var(input: &str) -> IResult<&str, Expr<&str>> {
    parse_identifier(input).map(|(rest, name)| (rest, Expr::Var { sym: name }))
}

#[cfg(test)]
mod tests {
    use crate::language::lvar::Expr;
    use crate::parser::var::parse_var;

    #[test]
    fn simple() {
        assert_eq!(parse_var("x").unwrap().1, Expr::Var { sym: "x" })
    }
}
