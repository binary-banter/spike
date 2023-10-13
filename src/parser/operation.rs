use crate::language::lvar::Op;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::IResult;
use nom::Parser;

pub fn parse_operation(input: &str) -> IResult<&str, Op> {
    alt((
        tag("+").map(|_| Op::Plus),
        tag("-").map(|_| Op::Minus),
        tag("*").map(|_| Op::Mul),
        tag("read").map(|_| Op::Read),
        tag("print").map(|_| Op::Print),
        tag("&&").map(|_| Op::LAnd),
        tag("||").map(|_| Op::LOr),
        tag("^").map(|_| Op::Xor),
        tag("<=").map(|_| Op::LE),
        tag("<").map(|_| Op::LT),
        tag("==").map(|_| Op::EQ),
        tag("!=").map(|_| Op::NE),
        tag("!").map(|_| Op::Not),
        tag(">=").map(|_| Op::GE),
        tag(">").map(|_| Op::GT),
    ))(input)
}
