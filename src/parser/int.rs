use crate::language::lvar::Expr;
use nom::character::complete::digit1;
use nom::combinator::map_res;
use nom::IResult;
use crate::interpreter::value::Val;

pub fn parse_int(input: &str) -> IResult<&str, Expr<&str>> {
    map_res(digit1, |n: &str| {
        n.parse::<i64>().map(|val| Expr::Val { val: Val::Int {val} })
    })(input)
}
