use crate::parser::re_find;
use nom::error::ErrorKind;
use nom::IResult;
use once_cell::sync::Lazy;
use regex::Regex;

static IDENTIFIER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z_]\w*").unwrap());
const RESERVED_KEYWORDS: [&str; 7] = ["t", "f", "fn", "if", "let", "Int", "Bool"];

pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
    let (rest, sym) = re_find(&IDENTIFIER_REGEX)(input)?;

    if RESERVED_KEYWORDS.contains(&sym) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            ErrorKind::Verify,
        )));
    }

    Ok((rest, sym))
}
