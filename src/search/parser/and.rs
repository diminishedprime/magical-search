use nom::{
    branch::alt, bytes::complete::tag_no_case, character::complete::space1, combinator::opt,
    multi::separated_list1, sequence::tuple, IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag};

use super::search_keyword;
use crate::search::ParsedSearch;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct And {
    pub items: Vec<ParsedSearch>,
    pub negated: bool,
}

pub fn and(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    tuple((
        opt(tag("-")),
        separated_list1(alt((tag_no_case(" AND "), space1)), search_keyword),
    ))
    .map(|(negate, items)| ParsedSearch::and(items, negate.is_some()))
    .parse(input)
}

impl ParsedSearch {
    pub fn and(searches: Vec<ParsedSearch>, negated: bool) -> Self {
        if searches.len() == 1 {
            searches
                .into_iter()
                .nth(0)
                .expect("Invalid invariant: Just checked length equals 1")
        } else {
            Self::And(And {
                items: searches,
                negated,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::search::Name;

    #[test]
    pub fn and_happy_path() {
        let input = "'name1' and 'name2'";
        let (_, actual) = and(input).unwrap();
        let expected = ParsedSearch::and(
            vec![
                ParsedSearch::name(Name::text("name1")),
                ParsedSearch::name(Name::text("name2")),
            ],
            false,
        );
        assert_eq!(actual, expected)
    }

    // #[test]
    // pub fn and_spelled_out_with_parens() {
    //     let input = "('name1' and 'name2')";
    //     let (_, actual) = and(input).unwrap();
    //     let expected = ParsedSearch::and(
    //         vec![
    //             ParsedSearch::name(Name::text("name1")),
    //             ParsedSearch::name(Name::text("name2")),
    //         ],
    //         false,
    //     );
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn and_not_spelled_out() {
    //     let input = "-('name1' 'name2')";
    //     let (_, actual) = and(input).unwrap();
    //     let expected = ParsedSearch::and(
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
    //     let (_, actual) = and(input).unwrap();
    //     let expected = ParsedSearch::and(
    //         vec![
    //             ParsedSearch::name(Name::text("name1")),
    //             ParsedSearch::name(Name::text("name2")),
    //         ],
    //         true,
    //     );
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn negated_and_spelled_out() {
    //     let input = "-('name1' and 'name2')";
    //     let (_, actual) = and(input).unwrap();
    //     let expected = ParsedSearch::and(
    //         vec![
    //             ParsedSearch::name(Name::text("name1")),
    //             ParsedSearch::name(Name::text("name2")),
    //         ],
    //         true,
    //     );
    //     assert_eq!(actual, expected)
    // }
}
