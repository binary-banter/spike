use crate::lvar::Expr;
use crate::parser::identifier::parse_identifier;
use nom::IResult;

pub fn parse_var(input: &str) -> IResult<&str, Expr> {
    parse_identifier(input).map(|(rest, name)| {
        (
            rest,
            Expr::Var {
                sym: name.to_string(),
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::lvar::Expr;
    use crate::parser::var::parse_var;

    #[test]
    fn simple() {
        assert_eq!(
            parse_var("x").unwrap().1,
            Expr::Var {
                sym: "x".to_string()
            }
        )
    }
}
