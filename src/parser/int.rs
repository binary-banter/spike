use crate::language::lvar::Expr;
use nom::character::complete::digit1;
use nom::combinator::map_res;
use nom::IResult;

pub fn parse_int(input: &str) -> IResult<&str, Expr<&str>> {
    map_res(digit1, |n: &str| {
        n.parse::<i64>().map(|val| Expr::Int { val })
    })(input)
}

#[cfg(test)]
mod tests {
    use crate::language::lvar::Expr;
    use crate::parser::int::parse_int;

    #[test]
    fn simple() {
        assert_eq!(parse_int("42").unwrap().1, Expr::Int { val: 42 })
    }
}
