use crate::parser::Operation;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::IResult;
use nom::Parser;

pub fn parse_operation(input: &str) -> IResult<&str, Operation> {
    alt((
        tag("+").map(|_| Operation::Plus),
        tag("-").map(|_| Operation::Minus),
        tag("read").map(|_| Operation::Read),
        tag("print").map(|_| Operation::Print),
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::operation::parse_operation;
    use crate::parser::Operation;

    #[test]
    fn add() {
        assert_eq!(parse_operation("+").unwrap().1, Operation::Plus)
    }
}
