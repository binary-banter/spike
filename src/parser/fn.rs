use crate::language::lvar::Def;
use crate::parser::expression::parse_expression;
use crate::parser::identifier::parse_identifier;
use crate::parser::r#type::parse_type;
use crate::parser::{trim0, trim1};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::multi::separated_list0;
use nom::sequence::{delimited, preceded, tuple};
use nom::{IResult, Parser};

pub fn parse_fn(input: &str) -> IResult<&str, Def<&str>> {
    preceded(
        trim0(tag("fn")),
        tuple((
            trim1(parse_identifier),
            delimited(
                trim0(char('(')),
                separated_list0(
                    trim0(char(',')),
                    tuple((
                        trim0(parse_identifier),
                        preceded(trim0(char(':')), trim0(parse_type)),
                    )),
                ),
                trim0(char(')')),
            ),
            preceded(trim0(tag("->")), trim0(parse_type)),
            delimited(trim0(char('{')), trim0(parse_expression), trim0(char('}'))),
        )),
    )
    .map(|(sym, args, typ, bdy)| Def::Fn {
        sym,
        args,
        typ,
        bdy,
    })
    .parse(input)
}
