use crate::parser::re_find;
use nom::IResult;
use once_cell::sync::Lazy;
use regex::Regex;

static IDENTIFIER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z_]\w*").unwrap());

pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
    re_find(&IDENTIFIER_REGEX)(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::identifier::parse_identifier;

    #[test]
    fn simple() {
        assert_eq!(parse_identifier("x").unwrap().1, "x")
    }

    #[test]
    fn trailing_whitespace() {
        assert_eq!(parse_identifier("x ").unwrap().1, "x")
    }

    #[test]
    fn underscore_prefix() {
        assert_eq!(parse_identifier("_x").unwrap().1, "_x")
    }

    #[test]
    fn underscore_postfix() {
        assert_eq!(parse_identifier("x_").unwrap().1, "x_")
    }

    #[test]
    fn underscore_middle() {
        assert_eq!(parse_identifier("x_y").unwrap().1, "x_y")
    }

    #[test]
    fn underscore_all() {
        assert_eq!(parse_identifier("_x_y_").unwrap().1, "_x_y_")
    }
}
