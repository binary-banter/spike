use crate::parser::Expression;
use nom::character::complete::digit1;
use nom::combinator::map_res;
use nom::IResult;

pub fn parse_int(input: &str) -> IResult<&str, Expression> {
    map_res(digit1, |n: &str| n.parse::<i64>().map(Expression::Int))(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::int::parse_int;
    use crate::parser::Expression;

    #[test]
    fn simple() {
        assert_eq!(parse_int("42").unwrap().1, Expression::Int(42))
    }
}
