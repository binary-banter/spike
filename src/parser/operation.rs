use crate::language::lvar::Op;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::IResult;
use nom::Parser;

pub fn parse_operation(input: &str) -> IResult<&str, Op> {
    alt((
        tag("+").map(|_| Op::Plus),
        tag("-").map(|_| Op::Minus),
        tag("read").map(|_| Op::Read),
        tag("print").map(|_| Op::Print),
        tag("&&").map(|_| Op::LAnd),
        tag("||").map(|_| Op::LOr),
        tag("!").map(|_| Op::Not),
        tag("^").map(|_| Op::Xor),
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::language::lvar::Op;
    use crate::parser::operation::parse_operation;

    #[test]
    fn add() {
        assert_eq!(parse_operation("+").unwrap().1, Op::Plus)
    }
}
