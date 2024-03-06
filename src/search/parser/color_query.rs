use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::opt,
    sequence::tuple,
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, ParserExt};

use super::{
    color::{color, ColorOperand},
    ParsedSearch,
};
use crate::search::SearchKeyword;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ColorOperator {
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
    pub operator: ColorOperator,
    pub operand: ColorOperand,
    pub is_negated: bool,
}

fn color_operator(input: &str) -> IResult<&str, ColorOperator, ErrorTree<&str>> {
    alt((
        tag("!=").value(ColorOperator::NotEqual),
        tag("<=").value(ColorOperator::LessThanOrEqual),
        tag(">=").value(ColorOperator::GreaterThanOrEqual),
        tag("<").value(ColorOperator::LessThan),
        tag(":").value(ColorOperator::Colon),
        tag("=").value(ColorOperator::Equal),
        tag(">").value(ColorOperator::GreaterThan),
    ))
    .parse(input)
}

pub fn color_query(input: &str) -> IResult<&str, ColorQuery, ErrorTree<&str>> {
    tuple((
        opt(tag("-")),
        alt((tag_no_case("color"), tag_no_case("c"))),
        color_operator,
        color,
    ))
    .map(|(negate, _color_tag, operator, comparison)| ColorQuery {
        operator,
        operand: comparison,
        is_negated: negate.is_some(),
    })
    .parse(input)
}

impl ParsedSearch {
    pub fn color_query(color: ColorQuery) -> Self {
        Self::Keyword(SearchKeyword::ColorQuery(color))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ColorQuery {
        fn not(self) -> Self {
            Self {
                operator: self.operator,
                operand: self.operand,
                is_negated: !self.is_negated,
            }
        }
        pub fn new(operator: ColorOperator, color: ColorOperand) -> Self {
            Self {
                operator,
                operand: color,
                is_negated: false,
            }
        }
    }

    #[test]
    fn test_color_query_lt_red() {
        let (_, actual) = color_query("c<red").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorOperator::LessThan, ColorOperand::Red)
        );
    }

    #[test]
    fn test_color_query_lte_green() {
        let (_, actual) = color_query("color<=green").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorOperator::LessThanOrEqual, ColorOperand::Green)
        );
    }

    #[test]
    fn test_color_query_gte_green_2() {
        let (_, actual) = color_query("color:green").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorOperator::Colon, ColorOperand::Green)
        );
    }

    #[test]
    fn test_color_query_is_blue() {
        let (_, actual) = color_query("c=blue").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorOperator::Equal, ColorOperand::Blue)
        );
    }

    #[test]
    fn test_color_query_gt_black() {
        let (_, actual) = color_query("color>black").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorOperator::GreaterThan, ColorOperand::Black)
        );
    }

    #[test]
    fn test_color_query_gte_white() {
        let (_, actual) = color_query("c>=white").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorOperator::GreaterThanOrEqual, ColorOperand::White)
        );
    }

    #[test]
    fn test_color_query_not_gte_white() {
        let (_, actual) = color_query("-c>=white").unwrap();
        assert_eq!(
            actual,
            ColorQuery::new(ColorOperator::GreaterThanOrEqual, ColorOperand::White).not()
        );
    }
}
