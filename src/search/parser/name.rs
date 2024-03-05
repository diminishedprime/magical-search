use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_while},
    sequence::delimited,
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag, ParserExt};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Name {
    pub text: String,
}

impl Name {
    pub fn text(s: &str) -> Self {
        Self {
            text: s.to_string(),
        }
    }
}

pub fn quoted_or_until_space(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((
        delimited(tag("'"), take_while(|c| c != '\''), tag("'")),
        delimited(tag("\""), take_while(|c| c != '"'), tag("\"")),
        take_while(|c| c != ' '),
    ))
    .parse(input)
}

pub fn name(input: &str) -> IResult<&str, Name, ErrorTree<&str>> {
    alt((tag_no_case("or"), tag_no_case("and")))
        .not()
        .peek()
        .parse(input)?;
    quoted_or_until_space.map(Name::text).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_single_quotes() {
        let (_, actual) = name("'this is my card name'").unwrap();
        assert_eq!(actual, Name::text("this is my card name"));
    }

    #[test]
    fn test_parse_name_double_quotes() {
        let (_, actual) = name(r#""this is also my card name""#).unwrap();
        assert_eq!(actual, Name::text("this is also my card name"));
    }

    #[test]
    fn test_parse_name_double_quotes_with_contraction() {
        let (_, actual) = name(r#""this isn't also my card name""#).unwrap();
        assert_eq!(actual, Name::text("this isn't also my card name"));
    }

    #[test]
    fn test_parse_standalone_name() {
        let (_, actual) = name("name").unwrap();
        assert_eq!(actual, Name::text("name"));
    }
}
