use nom::{branch::alt, sequence::delimited, IResult, Parser};
use nom_supreme::{error::ErrorTree, tag::complete::tag};

use super::{
    and::And,
    color_query::{color_query, ColorQuery},
    name,
    or::{or, Or},
    power_query::{power_query, PowerQuery},
    type_line_query::{type_line_query, TypeLineQuery},
    Name,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SearchKeyword {
    Color(ColorQuery),
    Power(PowerQuery),
    Name(Name),
    TypeLine(TypeLineQuery),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParsedSearch {
    Keyword(SearchKeyword),
    And(And),
    Or(Or),
}

pub fn search_keyword(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    fn base_parser(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
        alt((
            color_query.map(ParsedSearch::color),
            power_query.map(ParsedSearch::power),
            type_line_query.map(ParsedSearch::type_line),
            // Name must be the last parser since it's a bit of a catch-all.
            name.map(ParsedSearch::name),
        ))
        .parse(input)
    }

    alt((delimited(tag("("), base_parser, tag(")")), base_parser)).parse(input)
}

pub fn search(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    or.parse(input)
}

impl Default for ParsedSearch {
    fn default() -> Self {
        Self::and(vec![], false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{parser::color::Color, ColorComparison};

    impl ParsedSearch {
        fn leaf(keyword: SearchKeyword) -> Self {
            Self::Keyword(keyword)
        }
    }

    // #[test]
    // fn test_keyword_wrapped_in_parens() {
    //     let input = "(hello)";
    //     let (_, actual) = search(input).unwrap();
    //     assert_eq!(
    //         actual,
    //         ParsedSearch::leaf(SearchKeyword::Name(Name::text("hello")))
    //     );
    // }

    // #[test]
    // fn test_parse_search_single_color_query() {
    //     let input = "color:red";
    //     let (_, actual) = search(input).unwrap();
    //     assert_eq!(
    //         actual,
    //         ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
    //             ColorComparison::GreaterThanOrEqual,
    //             Color::Red
    //         )))
    //     );
    // }

    #[test]
    fn test_parse_search_multiple_implicit_and() {
        let input = "color=red color=blue color=green";
        let (_, actual) = search(input).unwrap();
        assert_eq!(
            actual,
            ParsedSearch::and(
                vec![
                    ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
                        ColorComparison::Equal,
                        Color::Red
                    ))),
                    ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
                        ColorComparison::Equal,
                        Color::Blue
                    ))),
                    ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
                        ColorComparison::Equal,
                        Color::Green
                    )))
                ],
                false
            )
        );
    }

    #[test]
    fn test_parse_search_multiple_and() {
        let input = "color=red AND color=blue AND color=green";
        let (_, actual) = search(input).unwrap();
        assert_eq!(
            actual,
            ParsedSearch::and(
                vec![
                    ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
                        ColorComparison::Equal,
                        Color::Red
                    ))),
                    ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
                        ColorComparison::Equal,
                        Color::Blue
                    ))),
                    ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
                        ColorComparison::Equal,
                        Color::Green
                    )))
                ],
                false
            )
        );
    }

    #[test]
    fn test_parse_search_multiple_or() {
        let input = "color=red OR color=blue OR color=green";
        let (_, actual) = search(input).unwrap();
        assert_eq!(
            actual,
            ParsedSearch::or(
                vec![
                    ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
                        ColorComparison::Equal,
                        Color::Red
                    ))),
                    ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
                        ColorComparison::Equal,
                        Color::Blue
                    ))),
                    ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
                        ColorComparison::Equal,
                        Color::Green
                    )))
                ],
                false
            )
        );
    }
}
