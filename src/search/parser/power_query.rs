use nom::{
    branch::alt, combinator::opt, number::complete::double, sequence::tuple, IResult, Parser,
};
use nom_supreme::{
    error::ErrorTree,
    tag::complete::{tag, tag_no_case},
    ParserExt,
};

use super::{comparison_operator, ComparisonOperator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerQuery {
    pub operator: ComparisonOperator,
    pub comparison: Comparison,
    pub is_negated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comparison {
    Number(String),
    Tougness,
}

impl Comparison {
    fn from_f64(num: f64) -> Self {
        Self::Number(num.to_string())
    }
}

pub fn power_query(input: &str) -> IResult<&str, PowerQuery, ErrorTree<&str>> {
    tuple((
        opt(tag("-")),
        alt((tag_no_case("power"), tag_no_case("pow"))),
        comparison_operator,
        power_comparison,
    ))
    .map(|(negate, _, operator, comparison)| PowerQuery {
        operator,
        comparison,
        is_negated: negate.is_some(),
    })
    .parse(input)
}

fn power_comparison(input: &str) -> IResult<&str, Comparison, ErrorTree<&str>> {
    alt((
        tag_no_case("toughness").value(Comparison::Tougness),
        tag_no_case("tou").value(Comparison::Tougness),
        double.map(Comparison::from_f64),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PowerQuery {
        fn not(self) -> Self {
            Self {
                operator: self.operator,
                comparison: self.comparison,
                is_negated: !self.is_negated,
            }
        }
        pub fn new(operator: ComparisonOperator, comparison: Comparison) -> Self {
            Self {
                operator,
                comparison,
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
                ComparisonOperator::GreaterThan,
                Comparison::Number("3".to_string()),
            )
        );
    }

    #[test]
    fn test_power_toughness() {
        let (_, actual) = power_query("power<=toughness").unwrap();
        assert_eq!(
            actual,
            PowerQuery::new(ComparisonOperator::LessThanOrEqual, Comparison::Tougness,)
        );
    }

    #[test]
    fn test_power_negated_toughness() {
        let (_, actual) = power_query("-pow>2.5").unwrap();
        assert_eq!(
            actual,
            PowerQuery::new(
                ComparisonOperator::GreaterThan,
                Comparison::Number("2.5".to_string()),
            )
            .not()
        );
    }
}
