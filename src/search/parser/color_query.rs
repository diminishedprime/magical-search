use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::opt,
    sequence::tuple,
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, ParserExt};

use super::color::{color, Color};
use crate::search::{ParsedSearch, SearchKeyword};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ColorComparison {
    LessThan,
    LessThanOrEqual,
    NotEqual,
    Colon,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
}

// Colors and Color Identity
// You can find cards that are a certain color using the c: or color: keyword,
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColorQuery {
    pub operator: ColorComparison,
    pub comparison: Color,
    pub is_negated: bool,
}

fn color_comparison(input: &str) -> IResult<&str, ColorComparison, ErrorTree<&str>> {
    alt((
        tag("!=").value(ColorComparison::NotEqual),
        tag("<=").value(ColorComparison::LessThanOrEqual),
        tag(">=").value(ColorComparison::GreaterThanOrEqual),
        tag("<").value(ColorComparison::LessThan),
        tag(":").value(ColorComparison::Colon),
        tag("=").value(ColorComparison::Equal),
        tag(">").value(ColorComparison::GreaterThan),
    ))
    .parse(input)
}

pub fn color_query(input: &str) -> IResult<&str, ColorQuery, ErrorTree<&str>> {
    tuple((
        opt(tag("-")),
        alt((tag_no_case("color"), tag_no_case("c"))),
        color_comparison,
        color,
    ))
    .map(|(negate, _color_tag, operator, comparison)| ColorQuery {
        operator,
        comparison,
        is_negated: negate.is_some(),
    })
    .parse(input)
}

impl ParsedSearch {
    pub fn color(color: ColorQuery) -> Self {
        Self::Keyword(SearchKeyword::Color(color))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ColorQuery {
        fn not(self) -> Self {
            Self {
                operator: self.operator,
                comparison: self.comparison,
                is_negated: !self.is_negated,
            }
        }
        pub fn new(operator: ColorComparison, color: Color) -> Self {
            Self {
                operator,
                comparison: color,
                is_negated: false,
            }
        }
    }

    #[test]
    fn test_color_query_lt_red() {
        let (_, actual) = color_query("c<red").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorComparison::LessThan, Color::Red)
        );
    }

    #[test]
    fn test_color_query_lte_green() {
        let (_, actual) = color_query("color<=green").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorComparison::LessThanOrEqual, Color::Green)
        );
    }

    #[test]
    fn test_color_query_gte_green_2() {
        let (_, actual) = color_query("color:green").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorComparison::Colon, Color::Green)
        );
    }

    #[test]
    fn test_color_query_is_blue() {
        let (_, actual) = color_query("c=blue").unwrap();
        assert_eq!(actual, ColorQuery::new(ColorComparison::Equal, Color::Blue));
    }

    #[test]
    fn test_color_query_gt_black() {
        let (_, actual) = color_query("color>black").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorComparison::GreaterThan, Color::Black)
        );
    }

    #[test]
    fn test_color_query_gte_white() {
        let (_, actual) = color_query("c>=white").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorComparison::GreaterThanOrEqual, Color::White)
        );
    }

    #[test]
    fn test_color_query_not_gte_white() {
        let (_, actual) = color_query("-c>=white").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorComparison::GreaterThanOrEqual, Color::White).not()
        );
    }
}
