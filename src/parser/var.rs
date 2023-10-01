use crate::parser::identifier::parse_identifier;
use crate::parser::Expression;
use nom::IResult;

pub fn parse_var(input: &str) -> IResult<&str, Expression> {
    parse_identifier(input).map(|(rest, name)| {
        (
            rest,
            Expression::Var {
                name: name.to_string(),
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::parser::var::parse_var;
    use crate::parser::Expression;

    #[test]
    fn simple() {
        assert_eq!(
            parse_var("x").unwrap().1,
            Expression::Var {
                name: "x".to_string()
            }
        )
    }
}
