use nom::{
    branch::alt, bytes::complete::tag_no_case, combinator::map, multi::separated_list1, IResult,
};

use super::color_query::{color_query, ColorQuery};

#[derive(Debug, PartialEq, Eq)]
pub enum SearchKeyword {
    Color(ColorQuery),
}

pub fn search_keyword(input: &str) -> IResult<&str, Search> {
    alt((map(color_query, |color| {
        Search::Keyword(SearchKeyword::Color(color))
    }),))(input)
}

fn or(input: &str) -> IResult<&str, Search> {
    map(separated_list1(tag_no_case(" OR "), and), |list| {
        if list.len() == 1 {
            list.into_iter()
                .nth(0)
                .expect("Invalid invariant: Just checked length equals 1")
        } else {
            Search::or(list)
        }
    })(input)
}

fn and(input: &str) -> IResult<&str, Search> {
    map(separated_list1(tag_no_case(" AND "), term), |list| {
        if list.len() == 1 {
            list.into_iter()
                .nth(0)
                .expect("Invalid invariant: Just checked length equals 1")
        } else {
            Search::and(list)
        }
    })(input)
}

fn term(input: &str) -> IResult<&str, Search> {
    let (rest, s) = search_keyword(input)?;
    Ok((rest, s))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Search {
    Keyword(SearchKeyword),
    And(Vec<Search>),
    Or(Vec<Search>),
}

impl Search {
    fn and(searches: Vec<Search>) -> Self {
        Self::And(searches)
    }
    fn or(searches: Vec<Search>) -> Self {
        Self::Or(searches)
    }
}

// TODO - use final parser here with nom_supreme
pub fn search(input: &str) -> IResult<&str, Search> {
    or(input)
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
        assert_eq!(
            search(input),
            Ok((
                "",
                Search::leaf(SearchKeyword::Color(ColorQuery::new(
                    ComparisonOperator::LessThanOrEqual,
                    Color::Red
                )))
            ))
        );
    }

    #[test]
    fn test_parse_search_multiple_and() {
        let input = "color=red AND color=blue AND color=green";
        assert_eq!(
            search(input),
            Ok((
                "",
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
            ))
        );
    }

    #[test]
    fn test_parse_search_multiple_or() {
        let input = "color=red OR color=blue OR color=green";
        assert_eq!(
            search(input),
            Ok((
                "",
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
            ))
        );
    }
}
