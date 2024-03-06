use std::fmt::{self, Display, Formatter};

use nom::{
    branch::alt, combinator::opt, number::complete::double, sequence::tuple, IResult, Parser,
};
use nom_supreme::{
    error::ErrorTree,
    tag::complete::{tag, tag_no_case},
    ParserExt,
};

use super::ParsedSearch;
use crate::search::SearchKeyword;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerOperator {
    LessThan,
    LessThanOrEqual,
    NotEqual,
    Colon,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
}

impl Display for PowerOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LessThan => write!(f, "<"),
            Self::LessThanOrEqual => write!(f, "<="),
            Self::NotEqual => write!(f, "!="),
            Self::Colon => write!(f, ":"),
            Self::Equal => write!(f, "="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterThanOrEqual => write!(f, ">="),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerQuery {
    pub operator: PowerOperator,
    pub operand: PowerOperand,
    pub is_negated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerOperand {
    Number(String),
    Tougness,
}

impl PowerOperand {
    fn from_f64(num: f64) -> Self {
        Self::Number(num.to_string())
    }
}

fn power_operator(input: &str) -> IResult<&str, PowerOperator, ErrorTree<&str>> {
    alt((
        tag("!=").value(PowerOperator::NotEqual),
        tag("<=").value(PowerOperator::LessThanOrEqual),
        tag(">=").value(PowerOperator::GreaterThanOrEqual),
        tag("<").value(PowerOperator::LessThan),
        tag(":").value(PowerOperator::Colon),
        tag("=").value(PowerOperator::Equal),
        tag(">").value(PowerOperator::GreaterThan),
    ))
    .parse(input)
}

fn power_operand(input: &str) -> IResult<&str, PowerOperand, ErrorTree<&str>> {
    alt((
        tag_no_case("toughness").value(PowerOperand::Tougness),
        tag_no_case("tou").value(PowerOperand::Tougness),
        double.map(PowerOperand::from_f64),
    ))
    .parse(input)
}

pub fn power_query(input: &str) -> IResult<&str, PowerQuery, ErrorTree<&str>> {
    tuple((
        opt(tag("-")),
        alt((tag_no_case("power"), tag_no_case("pow"))),
        power_operator,
        power_operand,
    ))
    .map(|(negate, _, operator, comparison)| PowerQuery {
        operator,
        operand: comparison,
        is_negated: negate.is_some(),
    })
    .parse(input)
}

impl ParsedSearch {
    pub fn power_query(power: PowerQuery) -> Self {
        Self::Keyword(SearchKeyword::PowerQuery(power))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PowerQuery {
        fn not(self) -> Self {
            Self {
                operator: self.operator,
                operand: self.operand,
                is_negated: !self.is_negated,
            }
        }
        pub fn new(operator: PowerOperator, comparison: PowerOperand) -> Self {
            Self {
                operator,
                operand: comparison,
                is_negated: false,
            }
        }
    }

    #[test]
    fn test_power_gt_3() {
        let (_, actual) = power_query("pow>3").unwrap();
        assert_eq!(
            actual,
            PowerQuery::new(
                PowerOperator::GreaterThan,
                PowerOperand::Number("3".to_string()),
            )
        );
    }

    #[test]
    fn test_power_toughness() {
        let (_, actual) = power_query("power<=toughness").unwrap();
        assert_eq!(
            actual,
            PowerQuery::new(PowerOperator::LessThanOrEqual, PowerOperand::Tougness,)
        );
    }

    #[test]
    fn test_power_negated_toughness() {
        let (_, actual) = power_query("-pow>2.5").unwrap();
        assert_eq!(
            actual,
            PowerQuery::new(
                PowerOperator::GreaterThan,
                PowerOperand::Number("2.5".to_string()),
            )
            .not()
        );
    }
}
