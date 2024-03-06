use nom::{
    combinator::opt,
    sequence::{delimited, tuple},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag};

use super::{parsed_search::parsed_search, ParsedSearch};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parens {
    pub operand: Box<ParsedSearch>,
    pub negated: bool,
}

pub fn parens(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    tuple((opt(tag("-")), delimited(tag("("), parsed_search, tag(")"))))
        .map(|(negate, items)| ParsedSearch::parens(items, negate.is_some()))
        .parse(input)
}

impl ParsedSearch {
    pub fn parens(operand: ParsedSearch, negated: bool) -> Self {
        Self::Parens(Parens {
            operand: Box::new(operand),
            negated,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::search::Name;

    #[test]
    pub fn parens_happy_path() {
        let input = "('name')";
        let (_, actual) = parens(input).unwrap();
        let expected = ParsedSearch::parens(ParsedSearch::name(Name::text("name", false)), false);
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn parens_happy_path_negated() {
        let input = "-('name')";
        let (_, actual) = parens(input).unwrap();
        let expected = ParsedSearch::parens(ParsedSearch::name(Name::text("name", false)), true);
        assert_eq!(actual, expected)
    }
}
