use nom::{
    branch::alt, bytes::complete::tag_no_case, character::complete::space1, multi::separated_list1,
    IResult, Parser,
};
use nom_supreme::error::ErrorTree;

use super::{
    color_query::{color_query, ColorQuery},
    name,
    power_query::{power_query, PowerQuery},
    Name,
};

#[derive(Debug, PartialEq, Eq)]
pub enum SearchKeyword {
    Color(ColorQuery),
    Power(PowerQuery),
    Name(Name),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Search {
    Keyword(SearchKeyword),
    And(Vec<Search>),
    Or(Vec<Search>),
}

pub fn search_keyword(input: &str) -> IResult<&str, Search, ErrorTree<&str>> {
    alt((
        color_query.map(Search::color),
        power_query.map(Search::power),
        name.map(Search::name),
    ))
    .parse(input)
}

fn and(input: &str) -> IResult<&str, Search, ErrorTree<&str>> {
    separated_list1(alt((tag_no_case(" AND "), space1)), search_keyword)
        .map(Search::and)
        .parse(input)
}

fn or(input: &str) -> IResult<&str, Search, ErrorTree<&str>> {
    separated_list1(tag_no_case(" OR "), and)
        .map(Search::or)
        .parse(input)
}

pub fn search(input: &str) -> IResult<&str, Search, ErrorTree<&str>> {
    or.parse(input)
}

impl Search {
    pub fn and(searches: Vec<Search>) -> Self {
        if searches.len() == 1 {
            searches
                .into_iter()
                .nth(0)
                .expect("Invalid invariant: Just checked length equals 1")
        } else {
            Self::And(searches)
        }
    }
    fn or(searches: Vec<Search>) -> Self {
        if searches.len() == 1 {
            searches
                .into_iter()
                .nth(0)
                .expect("Invalid invariant: Just checked length equals 1")
        } else {
            Self::Or(searches)
        }
    }
    fn color(color: ColorQuery) -> Self {
        Self::Keyword(SearchKeyword::Color(color))
    }
    fn power(power: PowerQuery) -> Self {
        Self::Keyword(SearchKeyword::Power(power))
    }
    fn name(name: Name) -> Self {
        Self::Keyword(SearchKeyword::Name(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::parser::{color::Color, comparison_operator::ComparisonOperator};

    impl Search {
        fn leaf(keyword: SearchKeyword) -> Self {
            Self::Keyword(keyword)
        }
    }

    #[test]
    fn test_parse_search_single_color_query() {
        let input = "color:red";
        let (_, actual) = search(input).unwrap();
        assert_eq!(
            actual,
            Search::leaf(SearchKeyword::Color(ColorQuery::new(
                ComparisonOperator::GreaterThanOrEqual,
                Color::Red
            )))
        );
    }

    #[test]
    fn test_parse_search_multiple_implicit_and() {
        let input = "color=red color=blue color=green";
        let (_, actual) = search(input).unwrap();
        assert_eq!(
            actual,
            Search::and(vec![
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::Equal,
                    Color::Red
                ))),
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::Equal,
                    Color::Blue
                ))),
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::Equal,
                    Color::Green
                )))
            ])
        );
    }

    #[test]
    fn test_parse_search_multiple_and() {
        let input = "color=red AND color=blue AND color=green";
        let (_, actual) = search(input).unwrap();
        assert_eq!(
            actual,
            Search::and(vec![
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::Equal,
                    Color::Red
                ))),
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::Equal,
                    Color::Blue
                ))),
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::Equal,
                    Color::Green
                )))
            ])
        );
    }

    #[test]
    fn test_parse_search_multiple_or() {
        let input = "color=red OR color=blue OR color=green";
        let (_, actual) = search(input).unwrap();
        assert_eq!(
            actual,
            Search::or(vec![
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::Equal,
                    Color::Red
                ))),
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::Equal,
                    Color::Blue
                ))),
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::Equal,
                    Color::Green
                )))
            ])
        );
    }
}
