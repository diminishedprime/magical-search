use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag};

use super::{
    and::{self, And},
    color_query::{color_query, ColorQuery},
    name,
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
    Or(Vec<ParsedSearch>),
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

fn or(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    alt((
        tuple((
            tag("("),
            separated_list1(tag_no_case(" OR "), and::and),
            tag(")"),
        ))
        .map(|(_, items, _)| items),
        separated_list1(tag_no_case(" OR "), and::and),
    ))
    .map(ParsedSearch::or)
    .parse(input)
}

pub fn search(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    or.parse(input)
}

impl Default for ParsedSearch {
    fn default() -> Self {
        Self::and(vec![], false)
    }
}

impl ParsedSearch {
    pub fn and(searches: Vec<ParsedSearch>, negated: bool) -> Self {
        if searches.len() == 1 {
            searches
                .into_iter()
                .nth(0)
                .expect("Invalid invariant: Just checked length equals 1")
        } else {
            Self::And(And {
                items: searches,
                negated,
            })
        }
    }
    fn or(searches: Vec<ParsedSearch>) -> Self {
        if searches.len() == 1 {
            searches
                .into_iter()
                .nth(0)
                .expect("Invalid invariant: Just checked length equals 1")
        } else {
            Self::Or(searches)
        }
    }
    pub fn color(color: ColorQuery) -> Self {
        Self::Keyword(SearchKeyword::Color(color))
    }
    pub fn power(power: PowerQuery) -> Self {
        Self::Keyword(SearchKeyword::Power(power))
    }
    pub fn name(name: Name) -> Self {
        Self::Keyword(SearchKeyword::Name(name))
    }
    pub fn type_line(type_line: TypeLineQuery) -> Self {
        Self::Keyword(SearchKeyword::TypeLine(type_line))
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
            ParsedSearch::or(vec![
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
            ])
        );
    }
}
