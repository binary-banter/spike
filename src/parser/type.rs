use crate::passes::type_check::Type;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::IResult;
use nom::Parser;

pub fn parse_type(input: &str) -> IResult<&str, Type> {
    alt((
        tag("Int").map(|_| Type::Int),
        tag("Bool").map(|_| Type::Bool),
    ))(input)
}
