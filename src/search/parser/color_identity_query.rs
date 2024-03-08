use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    sequence::tuple,
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, ParserExt};

use super::{
    color::{color, ColorOperand},
    parsed_search::SearchKeyword,
    ParsedSearch,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ColorIdentityOperator {
    LessThan,
    LessThanOrEqual,
    NotEqual,
    Colon,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColorIdentityQuery {
    pub operator: ColorIdentityOperator,
    pub operand: ColorOperand,
}

fn color_operator(input: &str) -> IResult<&str, ColorIdentityOperator, ErrorTree<&str>> {
    alt((
        tag("!=").value(ColorIdentityOperator::NotEqual),
        tag("<=").value(ColorIdentityOperator::LessThanOrEqual),
        tag(">=").value(ColorIdentityOperator::GreaterThanOrEqual),
        tag("<").value(ColorIdentityOperator::LessThan),
        tag(":").value(ColorIdentityOperator::Colon),
        tag("=").value(ColorIdentityOperator::Equal),
        tag(">").value(ColorIdentityOperator::GreaterThan),
    ))
    .parse(input)
}

pub fn color_identity_query(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    tuple((
        alt((tag_no_case("identity"), tag_no_case("id"))),
        color_operator,
        color,
    ))
    .map(|(_color_tag, operator, comparison)| ColorIdentityQuery {
        operator,
        operand: comparison,
    })
    .map(ParsedSearch::color_identity_query)
    .parse(input)
}

impl ParsedSearch {
    pub fn color_identity_query(color_identity_query: ColorIdentityQuery) -> Self {
        Self::Keyword(SearchKeyword::ColorIdentityQuery(color_identity_query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ColorIdentityQuery {
        pub fn new(operator: ColorIdentityOperator, color: ColorOperand) -> Self {
            Self {
                operator,
                operand: color,
            }
        }
    }

    #[test]
    fn test_color_identity_query_lt_red() {
        let (_, actual) = color_identity_query("c<red").unwrap();
        assert_eq!(
            actual,
            ParsedSearch::color_identity_query(ColorIdentityQuery::new(
                ColorIdentityOperator::LessThan,
                ColorOperand::Red
            ))
        );
    }

    #[test]
    fn test_color_identity_query_lte_green() {
        let (_, actual) = color_identity_query("color<=green").unwrap();
        assert_eq!(
            actual,
            ParsedSearch::color_identity_query(ColorIdentityQuery::new(
                ColorIdentityOperator::LessThanOrEqual,
                ColorOperand::Green
            ))
        );
    }

    #[test]
    fn test_color_identity_query_gte_green_2() {
        let (_, actual) = color_identity_query("color:green").unwrap();
        assert_eq!(
            actual,
            ParsedSearch::color_identity_query(ColorIdentityQuery::new(
                ColorIdentityOperator::Colon,
                ColorOperand::Green
            ))
        );
    }

    #[test]
    fn test_color_identity_query_is_blue() {
        let (_, actual) = color_identity_query("c=blue").unwrap();
        assert_eq!(
            actual,
            ParsedSearch::color_identity_query(ColorIdentityQuery::new(
                ColorIdentityOperator::Equal,
                ColorOperand::Blue
            ))
        );
    }

    #[test]
    fn test_color_identity_query_gt_black() {
        let (_, actual) = color_identity_query("color>black").unwrap();
        assert_eq!(
            actual,
            ParsedSearch::color_identity_query(ColorIdentityQuery::new(
                ColorIdentityOperator::GreaterThan,
                ColorOperand::Black
            ))
        );
    }

    #[test]
    fn test_color_identity_query_gte_white() {
        let (_, actual) = color_identity_query("c>=white").unwrap();
        assert_eq!(
            actual,
            ParsedSearch::color_identity_query(ColorIdentityQuery::new(
                ColorIdentityOperator::GreaterThanOrEqual,
                ColorOperand::White
            ))
        );
    }
}
