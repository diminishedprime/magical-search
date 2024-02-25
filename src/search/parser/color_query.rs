use {
    super::{
        color::{parse_color, Color},
        comparison_operator::{comparison_operator, ComparisonOperator},
    },
    nom::{
        branch::alt,
        bytes::complete::{tag, tag_no_case},
        combinator::opt,
        IResult,
    },
};

// Colors and Color Identity
// You can find cards that are a certain color using the c: or color: keyword,
#[derive(Debug, PartialEq, Eq)]
pub struct ColorQuery {
    pub operator: ComparisonOperator,
    pub color: Color,
    pub is_negated: bool,
}

pub fn color_query(input: &str) -> IResult<&str, ColorQuery> {
    let (input, negate) = opt(tag("-"))(input)?;
    let (input, _) = alt((tag_no_case("color"), tag_no_case("c")))(input)?;
    let (input, operator) = comparison_operator(input)?;
    let (input, color) = parse_color(input)?;
    Ok((
        input,
        ColorQuery {
            operator,
            color,
            is_negated: negate.is_some(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ColorQuery {
        fn not(self) -> Self {
            Self {
                operator: self.operator,
                color: self.color,
                is_negated: !self.is_negated,
            }
        }
        pub fn new(operator: ComparisonOperator, color: Color) -> Self {
            Self {
                operator,
                color,
                is_negated: false,
            }
        }
    }

    #[test]
    fn test_color_query_lt_red() {
        assert_eq!(
            color_query("c<red"),
            Ok((
                "",
                ColorQuery::new(ComparisonOperator::LessThan, Color::Red)
            ))
        );
    }

    #[test]
    fn test_color_query_lte_green() {
        assert_eq!(
            color_query("color<=green"),
            Ok((
                "",
                ColorQuery::new(ComparisonOperator::LessThanOrEqual, Color::Green)
            ))
        );
    }

    #[test]
    fn test_color_query_lte_green_2() {
        assert_eq!(
            color_query("color:green"),
            Ok((
                "",
                ColorQuery::new(ComparisonOperator::LessThanOrEqual, Color::Green)
            ))
        );
    }

    #[test]
    fn test_color_query_is_blue() {
        assert_eq!(
            color_query("c=blue"),
            Ok(("", ColorQuery::new(ComparisonOperator::Equal, Color::Blue)))
        );
    }

    #[test]
    fn test_color_query_gt_black() {
        assert_eq!(
            color_query("color>black"),
            Ok((
                "",
                ColorQuery::new(ComparisonOperator::GreaterThan, Color::Black)
            ))
        );
    }

    #[test]
    fn test_color_query_gte_white() {
        assert_eq!(
            color_query("c>=white"),
            Ok((
                "",
                ColorQuery::new(ComparisonOperator::GreaterThanOrEqual, Color::White)
            ))
        );
    }

    #[test]
    fn test_color_query_not_gte_white() {
        assert_eq!(
            color_query("-c>=white"),
            Ok((
                "",
                ColorQuery::new(ComparisonOperator::GreaterThanOrEqual, Color::White).not()
            ))
        );
    }
}
