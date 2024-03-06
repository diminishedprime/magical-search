use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_while},
    sequence::delimited,
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag, ParserExt};

use super::ParsedSearch;
use crate::search::SearchKeyword;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Name {
    pub text: String,
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

pub fn name(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    // TODO - Not sure if this is necessary anymore.
    alt((tag_no_case("or"), tag_no_case("and")))
        .not()
        .peek()
        .parse(input)?;
    quoted_or_until_space.map(ParsedSearch::name).parse(input)
}

impl ParsedSearch {
    pub fn name(name: &str) -> Self {
        Self::Keyword(SearchKeyword::Name(Name {
            text: name.to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_single_quotes() {
        let (_, actual) = name("'this is my card name'").unwrap();
        assert_eq!(actual, ParsedSearch::name("this is my card name"));
    }

    #[test]
    fn test_parse_name_double_quotes() {
        let (_, actual) = name(r#""this is also my card name""#).unwrap();
        assert_eq!(actual, ParsedSearch::name("this is also my card name"));
    }

    #[test]
    fn test_parse_name_double_quotes_with_contraction() {
        let (_, actual) = name(r#""this isn't also my card name""#).unwrap();
        assert_eq!(actual, ParsedSearch::name("this isn't also my card name"));
    }

    #[test]
    fn test_parse_standalone_name() {
        let (_, actual) = name("name").unwrap();
        assert_eq!(actual, ParsedSearch::name("name"));
    }

    #[test]
    fn test_parse_negated_name() {
        let (_, actual) = name("-name").unwrap();
        assert_eq!(actual, ParsedSearch::name("name"));
    }
}
