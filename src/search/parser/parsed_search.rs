use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::space1,
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag, ParserExt};

use super::{
    color_identity_query::{color_identity_query, ColorIdentityQuery},
    keyword::{keyword_query, KeywordQuery},
    name::Name,
    oracle_query::{oracle_query, OracleQuery},
    type_line_query::TypeLineQuery,
};
use crate::search::{
    color_query, name, power_query, type_line_query::type_line_query, ColorQuery, PowerQuery,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SearchKeyword {
    ColorQuery(ColorQuery),
    ColorIdentityQuery(ColorIdentityQuery),
    PowerQuery(PowerQuery),
    OracleQuery(OracleQuery),
    Name(Name),
    TypeLineQuery(TypeLineQuery),
    Keyword(KeywordQuery),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParsedSearch {
    Or(Vec<ParsedSearch>),
    And(Vec<ParsedSearch>),
    Negated(bool, Box<ParsedSearch>),
    Keyword(SearchKeyword),
}

pub fn parsed_search(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    or.parse(input)
}

fn or(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    separated_list0(alt((tag_no_case(" OR "), space1)), and)
        .map(ParsedSearch::Or)
        .parse(input)
}

fn and(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    separated_list0(alt((tag_no_case(" AND "), space1)), negated)
        .map(ParsedSearch::And)
        .parse(input)
}

fn negated(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    tuple((
        tag("-").opt(),
        alt((delimited(tag("("), parsed_search, tag(")")), search_keyword)),
    ))
    .map(|(negated, operand)| ParsedSearch::Negated(negated.is_some(), Box::new(operand)))
    .parse(input)
}

fn search_keyword(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    alt((
        color_query,
        color_identity_query,
        power_query,
        type_line_query,
        keyword_query,
        oracle_query,
        // Name must be the last parser since it's a bit of a catch-all.
        name,
    ))
    .parse(input)
}

#[cfg(test)]
mod test {
    use super::ParsedSearch;
    use crate::search::{
        color::ColorOperand, parsed_search::parsed_search, type_line_query::TypeLineQuery,
        ColorOperator, ColorQuery, PowerOperand, PowerOperator, PowerQuery,
    };

    fn test_or(inside: ParsedSearch) -> ParsedSearch {
        ParsedSearch::Or(vec![inside])
    }
    fn test_and(inside: ParsedSearch) -> ParsedSearch {
        ParsedSearch::And(vec![inside])
    }
    fn test_negated(negated: bool, inside: ParsedSearch) -> ParsedSearch {
        ParsedSearch::Negated(negated, Box::new(inside))
    }

    #[test]
    fn test_keyword_wrapped_in_parens() {
        let input = "(hello)";
        let (_, actual) = parsed_search(input).unwrap();
        let expected = test_or(test_and(test_negated(
            false,
            test_or(test_and(test_negated(false, ParsedSearch::name("hello")))),
        )));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_negated_keyword_wrapped_in_parens() {
        let input = "(-hello)";
        let (_, actual) = parsed_search(input).unwrap();
        let expected = test_or(test_and(test_negated(
            false,
            test_or(test_and(test_negated(true, ParsedSearch::name("hello")))),
        )));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_search_single_color_query() {
        let input = "color:red";
        let (_, actual) = parsed_search(input).unwrap();

        let expected = test_or(test_and(test_negated(
            false,
            ParsedSearch::color_query(ColorQuery {
                operand: ColorOperand::Red,
                operator: ColorOperator::Colon,
            }),
        )));
        assert_eq!(actual, expected);
    }

    #[test]
    fn basic_commander_search() {
        let input = "c>=esper pow<3 t:creature";
        let expected = test_or(ParsedSearch::And(vec![
            test_negated(
                false,
                ParsedSearch::color_query(ColorQuery {
                    operator: ColorOperator::GreaterThanOrEqual,
                    operand: ColorOperand::Esper,
                }),
            ),
            test_negated(
                false,
                ParsedSearch::power_query(PowerQuery {
                    operator: PowerOperator::LessThan,
                    operand: PowerOperand::Number("3".to_string()),
                    negated: false,
                }),
            ),
            test_negated(
                false,
                ParsedSearch::type_line(TypeLineQuery {
                    operand: "creature".to_string(),
                    negated: false,
                }),
            ),
        ]));
        let (_, actual) = parsed_search(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    pub fn top_level_parens_and() {
        let input = "(c:esper pow<3)";
        let expected = test_or(test_and(test_negated(
            false,
            test_or(ParsedSearch::And(vec![
                test_negated(
                    false,
                    ParsedSearch::color_query(ColorQuery {
                        operator: ColorOperator::Colon,
                        operand: ColorOperand::Esper,
                    }),
                ),
                test_negated(
                    false,
                    ParsedSearch::power_query(PowerQuery {
                        operator: PowerOperator::LessThan,
                        operand: PowerOperand::Number("3".to_string()),
                        negated: false,
                    }),
                ),
            ])),
        )));
        let (_, actual) = parsed_search(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    pub fn top_level_parens_one_name() {
        let input = "(sliver)";
        let expected = test_or(test_and(test_negated(
            false,
            test_or(test_and(test_negated(false, ParsedSearch::name("sliver")))),
        )));
        let (_, actual) = parsed_search(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    pub fn top_level_parens_two_names() {
        let input = "(sliver queen)";
        let expected = test_or(test_and(test_negated(
            false,
            test_or(ParsedSearch::And(vec![
                test_negated(false, ParsedSearch::name("sliver")),
                test_negated(false, ParsedSearch::name("queen")),
            ])),
        )));
        let (_, actual) = parsed_search(input).unwrap();
        assert_eq!(actual, expected);
    }
}
