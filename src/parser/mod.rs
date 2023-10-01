use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1, multispace0, multispace1};
use nom::combinator::{all_consuming, map_res};
use nom::error::{ErrorKind, ParseError};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::Err;
use nom::{IResult, InputLength, Parser, Slice};
use once_cell::sync::Lazy;
use regex::Regex;

#[allow(unused)]
pub enum Type {
    Integer,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Read,
    Print,
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
pub enum Expression<'input> {
    Int(i64),
    Var {
        name: &'input str,
    },
    Prim {
        operation: Operation,
        arguments: Vec<Expression<'input>>,
    },
    Let {
        name: &'input str,
        binding: Box<Expression<'input>>,
        body: Box<Expression<'input>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct LVarProgram<'input> {
    body: Expression<'input>,
}

pub fn re_match<'a, 'b, E>(re: &'b Regex) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, E> + 'b
where
    E: ParseError<&'a str>,
{
    move |i| {
        if re.is_match(i) {
            Ok((i.slice(i.input_len()..), i))
        } else {
            Err(Err::Error(E::from_error_kind(i, ErrorKind::RegexpMatch)))
        }
    }
}

static IDENTIFIER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*").unwrap());

pub fn parse_program<'input>(input: &'input str) -> IResult<&str, LVarProgram<'input>> {
    all_consuming(terminated(parse_expression, multispace0))
        .map(|body| LVarProgram { body })
        .parse(input)
}

pub fn parse_expression<'input>(input: &'input str) -> IResult<&str, Expression<'input>> {
    alt((parse_int, parse_var, parse_prim, parse_let))(input)
}

pub fn parse_int<'input>(input: &'input str) -> IResult<&str, Expression<'input>> {
    map_res(digit1, |n: &str| n.parse::<i64>().map(Expression::Int))(input)
}

pub fn parse_identifier<'input>(input: &'input str) -> IResult<&str, &str> {
    re_match(&*IDENTIFIER_REGEX)(input)
}

pub fn parse_var<'input>(input: &'input str) -> IResult<&str, Expression<'input>> {
    parse_identifier(input).map(|(rest, name)| (rest, Expression::Var { name }))
}

pub fn parse_prim<'input>(input: &'input str) -> IResult<&str, Expression<'input>> {
    delimited(
        char('('),
        pair(trim0(parse_operation), many0(trim1(parse_expression))).map(
            |(operation, arguments)| Expression::Prim {
                operation,
                arguments,
            },
        ),
        trim0(char(')')),
    )(input)
}

pub fn parse_let<'input>(input: &'input str) -> IResult<&str, Expression<'input>> {
    delimited(
        char('('),
        tuple((
            trim0(tag("let")),
            trim1(delimited(
                char('('),
                pair(parse_identifier, parse_expression),
                char(')'),
            )),
            trim1(parse_expression),
        )),
        trim0(char(')')),
    )
    .map(|(_, (name, binding), body)| Expression::Let {
        name,
        binding: Box::new(binding),
        body: Box::new(body),
    })
    .parse(input)
}

pub fn trim0<'a, O, E: ParseError<&'a str>>(
    parser: impl Parser<&'a str, O, E>,
) -> impl Parser<&'a str, O, E> {
    preceded(multispace0, parser)
}

pub fn trim1<'a, O, E: ParseError<&'a str>>(
    parser: impl Parser<&'a str, O, E>,
) -> impl Parser<&'a str, O, E> {
    preceded(multispace1, parser)
}

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
    use crate::parser::{parse_expression, parse_program, Expression, LVarProgram, Operation};

    #[test]
    fn int() {
        assert!(matches!(
            parse_program("1").unwrap().1,
            LVarProgram {
                body: Expression::Int(1)
            }
        ))
    }

    #[test]
    fn var() {
        assert!(matches!(
            parse_program("x").unwrap().1,
            LVarProgram {
                body: Expression::Var { name: "x" }
            }
        ))
    }

    #[test]
    fn add() {
        assert_eq!(
            parse_program("(+ 10 32)").unwrap().1,
            LVarProgram {
                body: Expression::Prim {
                    operation: Operation::Plus,
                    arguments: vec![Expression::Int(10), Expression::Int(32)]
                }
            }
        )
    }

    #[test]
    fn let_binding() {
        assert_eq!(
            parse_program("(let (x 42) x)").unwrap().1,
            LVarProgram {
                body: Expression::Let {
                    name: "x",
                    binding: Box::new(Expression::Int(42)),
                    body: Box::new(Expression::Var { name: "x" })
                }
            }
        )
    }
}
