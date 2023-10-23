use crate::language::lvar::Op;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::IResult;
use nom::Parser;

pub fn parse_operation(input: &str) -> IResult<&str, Op> {
    alt((
        tag("read").map(|_| Op::Read),
        tag("print").map(|_| Op::Print),
        tag("&&").map(|_| Op::LAnd),
        tag("||").map(|_| Op::LOr),
        tag("<=").map(|_| Op::LE),
        tag("==").map(|_| Op::EQ),
        tag("!=").map(|_| Op::NE),
        tag(">=").map(|_| Op::GE),
        char('+').map(|_| Op::Plus),
        char('-').map(|_| Op::Minus),
        char('*').map(|_| Op::Mul),
        char('/').map(|_| Op::Div),
        char('%').map(|_| Op::Mod),
        char('<').map(|_| Op::LT),
        char('!').map(|_| Op::Not),
        char('^').map(|_| Op::Xor),
        char('>').map(|_| Op::GT),
    ))(input)
}
