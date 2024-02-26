use nom::{branch::alt, bytes::complete::tag, IResult, Parser};
use nom_supreme::{error::ErrorTree, ParserExt};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    NotEqual,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
}

impl ComparisonOperator {
    pub fn negate(&self) -> ComparisonOperator {
        match self {
            ComparisonOperator::LessThan => ComparisonOperator::GreaterThanOrEqual,
            ComparisonOperator::LessThanOrEqual => ComparisonOperator::GreaterThan,
            ComparisonOperator::NotEqual => ComparisonOperator::Equal,
            ComparisonOperator::Equal => ComparisonOperator::NotEqual,
            ComparisonOperator::GreaterThan => ComparisonOperator::LessThanOrEqual,
            ComparisonOperator::GreaterThanOrEqual => ComparisonOperator::LessThan,
        }
    }
}

pub fn comparison_operator(input: &str) -> IResult<&str, ComparisonOperator, ErrorTree<&str>> {
    alt((
        tag("!=").value(ComparisonOperator::NotEqual),
        tag("<=").value(ComparisonOperator::LessThanOrEqual),
        tag(">=").value(ComparisonOperator::GreaterThanOrEqual),
        tag("<").value(ComparisonOperator::LessThan),
        tag(":").value(ComparisonOperator::GreaterThanOrEqual),
        tag("=").value(ComparisonOperator::Equal),
        tag(">").value(ComparisonOperator::GreaterThan),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comparison_operator_less_than() {
        let (_, actual) = comparison_operator("<").unwrap();
        assert_eq!(actual, ComparisonOperator::LessThan);
    }

    #[test]
    fn test_parse_comparison_operator_less_than_or_equal() {
        let (_, actual) = comparison_operator("<=").unwrap();
        assert_eq!(actual, ComparisonOperator::LessThanOrEqual);
        let (_, actual) = comparison_operator(":").unwrap();
        assert_eq!(actual, ComparisonOperator::LessThanOrEqual);
    }

    #[test]
    fn test_parse_comparison_operator_equal() {
        let (_, actual) = comparison_operator("=").unwrap();
        assert_eq!(actual, ComparisonOperator::Equal);
    }

    #[test]
    fn test_parse_comparison_operator_not_equal() {
        let (_, actual) = comparison_operator("!=").unwrap();
        assert_eq!(actual, ComparisonOperator::NotEqual);
    }

    #[test]
    fn test_parse_comparison_operator_greater_than() {
        let (_, actual) = comparison_operator(">").unwrap();
        assert_eq!(actual, ComparisonOperator::GreaterThan);
    }

    #[test]
    fn test_parse_comparison_operator_greater_than_or_equal() {
        let (_, actual) = comparison_operator(">=").unwrap();
        assert_eq!(actual, ComparisonOperator::GreaterThanOrEqual);
    }
}
