use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_while},
    combinator::opt,
    sequence::{delimited, tuple},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag, ParserExt};

use super::ParsedSearch;
use crate::search::SearchKeyword;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Name {
    pub text: String,
    pub negated: bool,
}

impl Name {
    pub fn text(s: &str, negated: bool) -> Self {
        Self {
            text: s.to_string(),
            negated,
        }
    }
}

pub fn quoted_or_until_space(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    // Trying to make this fail if it's actually parsing a parens.
    alt((
        delimited(tag("'"), take_while(|c| c != '\''), tag("'")),
        delimited(tag("\""), take_while(|c| c != '"'), tag("\"")),
        take_while(|c| c != ' ' && c != ')'),
    ))
    .parse(input)
}

pub fn name(input: &str) -> IResult<&str, Name, ErrorTree<&str>> {
    alt((tag_no_case("or"), tag_no_case("and")))
        .not()
        .peek()
        .parse(input)?;
    tuple((opt(tag("-")), quoted_or_until_space))
        .map(|(negated, text)| Name::text(text, negated.is_some()))
        .parse(input)
}

impl ParsedSearch {
    pub fn name(name: Name) -> Self {
        Self::Keyword(SearchKeyword::Name(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_single_quotes() {
        let (_, actual) = name("'this is my card name'").unwrap();
        assert_eq!(actual, Name::text("this is my card name", false));
    }

    #[test]
    fn test_parse_name_double_quotes() {
        let (_, actual) = name(r#""this is also my card name""#).unwrap();
        assert_eq!(actual, Name::text("this is also my card name", false));
    }

    #[test]
    fn test_parse_name_double_quotes_with_contraction() {
        let (_, actual) = name(r#""this isn't also my card name""#).unwrap();
        assert_eq!(actual, Name::text("this isn't also my card name", false));
    }

    #[test]
    fn test_parse_standalone_name() {
        let (_, actual) = name("name").unwrap();
        assert_eq!(actual, Name::text("name", false));
    }

    #[test]
    fn test_parse_negated_name() {
        let (_, actual) = name("-name").unwrap();
        assert_eq!(actual, Name::text("name", true));
    }
}
