use nom::{IResult, Parser};
use nom_supreme::error::ErrorTree;

use super::{
    and::And,
    or::{or, Or},
};
use crate::search::SearchKeyword;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParsedSearch {
    Keyword(SearchKeyword),
    And(And),
    Or(Or),
}

pub fn parsed_search(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    or.parse(input)
}

impl Default for ParsedSearch {
    fn default() -> Self {
        Self::and(vec![], false)
    }
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn test_keyword_wrapped_in_parens() {
    //     let input = "(hello)";
    //     let (_, actual) = search(input).unwrap();
    //     assert_eq!(
    //         actual,
    //         ParsedSearch::leaf(SearchKeyword::Name(Name::text("hello")))
    //     );
    // }

    // #[test]
    // fn test_parse_search_single_color_query() {
    //     let input = "color:red";
    //     let (_, actual) = search(input).unwrap();
    //     assert_eq!(
    //         actual,
    //         ParsedSearch::leaf(SearchKeyword::Color(ColorQuery::new(
    //             ColorComparison::GreaterThanOrEqual,
    //             Color::Red
    //         )))
    //     );
    // }
}
