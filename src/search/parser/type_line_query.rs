use nom::{branch::alt, combinator::opt, sequence::tuple, IResult, Parser};
use nom_supreme::{
    error::ErrorTree,
    tag::complete::{tag, tag_no_case},
};

use super::quoted_or_until_space;
use crate::search::{ParsedSearch, SearchKeyword};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeLineQuery {
    pub comparison: String,
    pub is_negated: bool,
}

pub fn type_line_query(input: &str) -> IResult<&str, TypeLineQuery, ErrorTree<&str>> {
    tuple((
        opt(tag("-")),
        alt((tag_no_case("type"), tag_no_case("t"))),
        alt((tag(":"), tag("="))),
        quoted_or_until_space,
    ))
    .map(|(negate, _, _, comparison)| TypeLineQuery {
        comparison: comparison.to_string(),
        is_negated: negate.is_some(),
    })
    .parse(input)
}

impl ParsedSearch {
    pub fn type_line(type_line: TypeLineQuery) -> Self {
        Self::Keyword(SearchKeyword::TypeLine(type_line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_line_query_type() {
        let input = r#"type:"Creature - Goblin""#;
        let expected = TypeLineQuery {
            comparison: "Creature - Goblin".to_string(),
            is_negated: false,
        };
        let (_, actual) = type_line_query(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_type_line_query_negated_type() {
        let input = r#"-type:"Sorcery""#;
        let expected = TypeLineQuery {
            comparison: "Sorcery".to_string(),
            is_negated: true,
        };
        let (_, actual) = type_line_query(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_type_line_query_t() {
        let input = r#"t:"Land""#;
        let expected = TypeLineQuery {
            comparison: "Land".to_string(),
            is_negated: false,
        };
        let (_, actual) = type_line_query(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_type_line_query_negated_t() {
        let input = r#"-t:"Enchantment Creature - Human""#;
        let expected = TypeLineQuery {
            comparison: "Enchantment Creature - Human".to_string(),
            is_negated: true,
        };
        let (_, actual) = type_line_query(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_type_line_query_equals_sign() {
        let input = r#"type=Artifact"#;
        let expected = TypeLineQuery {
            comparison: "Artifact".to_string(),
            is_negated: false,
        };
        let (_, actual) = type_line_query(input).unwrap();
        assert_eq!(actual, expected);
    }
}
