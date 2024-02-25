use nom::{branch::alt, bytes::complete::tag, IResult, Parser};
use nom_supreme::ParserExt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
}

pub fn comparison_operator(input: &str) -> IResult<&str, ComparisonOperator> {
    alt((
        tag("<=").value(ComparisonOperator::LessThanOrEqual),
        tag(">=").value(ComparisonOperator::GreaterThanOrEqual),
        tag("<").value(ComparisonOperator::LessThan),
        tag(":").value(ComparisonOperator::LessThanOrEqual),
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
        assert_eq!(
            comparison_operator("<"),
            Ok(("", ComparisonOperator::LessThan))
        );
    }

    #[test]
    fn test_parse_comparison_operator_less_than_or_equal() {
        assert_eq!(
            comparison_operator("<="),
            Ok(("", ComparisonOperator::LessThanOrEqual))
        );
        assert_eq!(
            comparison_operator(":"),
            Ok(("", ComparisonOperator::LessThanOrEqual))
        );
    }

    #[test]
    fn test_parse_comparison_operator_equal() {
        assert_eq!(
            comparison_operator("="),
            Ok(("", ComparisonOperator::Equal))
        );
    }

    #[test]
    fn test_parse_comparison_operator_greater_than() {
        assert_eq!(
            comparison_operator(">"),
            Ok(("", ComparisonOperator::GreaterThan))
        );
    }

    #[test]
    fn test_parse_comparison_operator_greater_than_or_equal() {
        assert_eq!(
            comparison_operator(">="),
            Ok(("", ComparisonOperator::GreaterThanOrEqual))
        );
    }
}
