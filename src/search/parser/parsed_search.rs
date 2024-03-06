use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::space1,
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag, ParserExt};

use crate::search::{
    color_query, name, power_query, type_line_query::type_line_query, SearchKeyword,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParsedSearch {
    Or(Vec<ParsedSearch>),
    And(Vec<ParsedSearch>),
    Negated(bool, Box<ParsedSearch>),
    Keyword(SearchKeyword),
}

pub fn parsed_search(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    or.parse(input)
}

fn or(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    separated_list0(alt((tag_no_case(" OR "), space1)), and)
        .map(ParsedSearch::Or)
        .parse(input)
}

fn and(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    separated_list0(alt((tag_no_case(" AND "), space1)), negated)
        .map(ParsedSearch::And)
        .parse(input)
}

fn negated(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    tuple((
        tag("-").opt(),
        alt((delimited(tag("("), parsed_search, tag(")")), search_keyword)),
    ))
    .map(|(negated, operand)| ParsedSearch::Negated(negated.is_some(), Box::new(operand)))
    .parse(input)
}

fn search_keyword(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    alt((
        color_query,
        power_query,
        type_line_query,
        // Name must be the last parser since it's a bit of a catch-all.
        name,
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use crate::search::{
        color::ColorOperand::Red, parsed_search::parsed_search, ColorOperator::Colon, ColorQuery,
        ParsedSearch,
    };

    #[test]
    fn test_keyword_wrapped_in_parens() {
        let input = "(hello)";
        let (_, actual) = parsed_search(input).unwrap();
        let expected = ParsedSearch::name("hello");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_negated_keyword_wrapped_in_parens() {
        let input = "(-hello)";
        let (_, actual) = parsed_search(input).unwrap();
        let expected = ParsedSearch::name("hello");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_search_single_color_query() {
        let input = "color:red";
        let (_, actual) = parsed_search(input).unwrap();
        let expected = ParsedSearch::color_query(ColorQuery {
            operand: Red,
            operator: Colon,
            negated: false,
        });
        assert_eq!(actual, expected);
    }
}
