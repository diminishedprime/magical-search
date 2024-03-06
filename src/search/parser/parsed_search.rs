use nom::{branch::alt, IResult, Parser};
use nom_supreme::error::ErrorTree;

use super::{
    and::And,
    or::{or, Or},
    parens::{parens, Parens},
};
use crate::search::{search_keyword, SearchKeyword};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParsedSearch {
    Keyword(SearchKeyword),
    And(And),
    Or(Or),
    Parens(Parens),
}

impl ParsedSearch {
    pub fn negated(mut self, negated: bool) -> Self {
        match &mut self {
            ParsedSearch::Keyword(ref mut kw) => match kw {
                SearchKeyword::ColorQuery(ref mut cq) => cq.negated = negated,
                SearchKeyword::PowerQuery(ref mut pq) => pq.negated = negated,
                SearchKeyword::Name(ref mut name) => name.negated = negated,
                SearchKeyword::TypeLineQuery(ref mut tlq) => tlq.negated = negated,
            },
            ParsedSearch::And(ref mut and) => and.negated = negated,
            ParsedSearch::Or(ref mut or) => or.negated = negated,
            ParsedSearch::Parens(ref mut parens) => parens.negated = negated,
        }
        self
    }
}

pub fn parsed_search(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    alt((parens, or, search_keyword)).parse(input)
}

#[cfg(test)]
mod tests {
    use crate::search::{
        color::ColorOperand::Red, parsed_search::parsed_search, ColorOperator::Colon, ColorQuery,
        Name, ParsedSearch,
    };

    #[test]
    fn test_keyword_wrapped_in_parens() {
        let input = "(hello)";
        let (_, actual) = parsed_search(input).unwrap();
        assert_eq!(
            actual,
            ParsedSearch::parens(ParsedSearch::name(Name::text("hello", false)), false)
        );
    }

    #[test]
    fn test_negated_keyword_wrapped_in_parens() {
        let input = "(-hello)";
        let (_, actual) = parsed_search(input).unwrap();
        assert_eq!(
            actual,
            ParsedSearch::parens(ParsedSearch::name(Name::text("hello", true)), false)
        );
    }

    #[test]
    fn test_parse_search_single_color_query() {
        let input = "color:red";
        let (_, actual) = parsed_search(input).unwrap();
        let expected = ParsedSearch::color_query(ColorQuery {
            operand: Red,
            operator: Colon,
            negated: false,
        });
        assert_eq!(actual, expected);
    }
}
