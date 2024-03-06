use nom::{
    branch::alt, bytes::complete::tag_no_case, character::complete::space1, combinator::opt,
    multi::separated_list0, sequence::tuple, IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag};

use super::{and::and, ParsedSearch};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Or {
    pub operands: Vec<ParsedSearch>,
    pub negated: bool,
}

pub fn or(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    tuple((
        opt(tag("-")),
        separated_list0(alt((tag_no_case(" OR "), space1)), and),
    ))
    .map(|(negate, items)| ParsedSearch::or(items, negate.is_some()))
    .parse(input)
}

impl ParsedSearch {
    pub fn or(operands: Vec<ParsedSearch>, negated: bool) -> Self {
        if operands.len() == 1 {
            operands
                .into_iter()
                .nth(0)
                .expect("Invalid invariant: Just checked length equals 1")
                .negated(negated)
        } else {
            Self::Or(Or { operands, negated })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::search::Name;

    #[test]
    pub fn or_happy_path() {
        let input = "'name1' or 'name2'";
        let (_, actual) = or(input).unwrap();
        let expected = ParsedSearch::or(
            vec![
                ParsedSearch::name(Name::text("name1", false)),
                ParsedSearch::name(Name::text("name2", false)),
            ],
            false,
        );
        assert_eq!(actual, expected)
    }

    // #[test]
    // pub fn or_spelled_out_with_parens() {
    //     let input = "('name1' or 'name2')";
    //     let (_, actual) = or(input).unwrap();
    //     let expected = ParsedSearch::or(
    //         vec![
    //             ParsedSearch::name(Name::text("name1")),
    //             ParsedSearch::name(Name::text("name2")),
    //         ],
    //         false,
    //     );
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn or_not_spelled_out() {
    //     let input = "-('name1' 'name2')";
    //     let (_, actual) = or(input).unwrap();
    //     let expected = ParsedSearch::or(
    //         vec![
    //             ParsedSearch::name(Name::text("name1")),
    //             ParsedSearch::name(Name::text("name2")),
    //         ],
    //         true,
    //     );
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn negated_not_spelled_out() {
    //     let input = "-('name1' 'name2')";
    //     let (_, actual) = or(input).unwrap();
    //     let expected = ParsedSearch::or(
    //         vec![
    //             ParsedSearch::name(Name::text("name1")),
    //             ParsedSearch::name(Name::text("name2")),
    //         ],
    //         true,
    //     );
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn negated_or_spelled_out() {
    //     let input = "-('name1' or 'name2')";
    //     let (_, actual) = or(input).unwrap();
    //     let expected = ParsedSearch::or(
    //         vec![
    //             ParsedSearch::name(Name::text("name1")),
    //             ParsedSearch::name(Name::text("name2")),
    //         ],
    //         true,
    //     );
    //     assert_eq!(actual, expected)
    // }
}
