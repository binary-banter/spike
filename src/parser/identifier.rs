use crate::parser::re_find;
use nom::IResult;
use once_cell::sync::Lazy;
use regex::Regex;

static IDENTIFIER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z_]\w*").unwrap());

pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
    re_find(&IDENTIFIER_REGEX)(input)
}
