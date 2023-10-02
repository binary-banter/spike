use crate::parser::Expr;
use nom::character::complete::digit1;
use nom::combinator::map_res;
use nom::IResult;

pub fn parse_int(input: &str) -> IResult<&str, Expr> {
    map_res(digit1, |n: &str| n.parse::<i64>().map(Expr::Int))(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::int::parse_int;
    use crate::parser::Expr;

    #[test]
    fn simple() {
        assert_eq!(parse_int("42").unwrap().1, Expr::Int(42))
    }
}
