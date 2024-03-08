use nom::{branch::alt, sequence::tuple, IResult, Parser};
use nom_supreme::{error::ErrorTree, tag::complete::tag_no_case};

use super::{name::quoted_or_until_space, ParsedSearch};
use crate::search::SearchKeyword;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordQuery {
    pub keyword: String,
}

fn keyword(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    alt((
        tag_no_case("doublestrike").map(|_| "Double Strike"),
        quoted_or_until_space,
    ))
    .parse(input)
}

pub fn keyword_query(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    tuple((alt((tag_no_case("kw:"), tag_no_case("keyword:"))), keyword))
        .map(|(_, keyword)| KeywordQuery {
            keyword: keyword.to_string(),
        })
        .map(ParsedSearch::keyword_query)
        .parse(input)
}

impl ParsedSearch {
    pub fn keyword_query(keyword: KeywordQuery) -> Self {
        Self::Keyword(SearchKeyword::Keyword(keyword))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_strike() {
        let (_, actual) = keyword_query("keyword:doublestrike").unwrap();
        assert_eq!(
            actual,
            super::ParsedSearch::keyword_query(super::KeywordQuery {
                keyword: "Double Strike".to_string()
            })
        );
    }

    #[test]
    fn test_double_strike_spell_out() {
        let (_, actual) = keyword_query(r#"keyword:"double strike""#).unwrap();
        assert_eq!(
            actual,
            super::ParsedSearch::keyword_query(super::KeywordQuery {
                keyword: "double strike".to_string()
            })
        );
    }

    #[test]
    fn test_flying_kw() {
        let (_, actual) = keyword_query("kw:flying").unwrap();
        assert_eq!(
            actual,
            super::ParsedSearch::keyword_query(super::KeywordQuery {
                keyword: "flying".to_string()
            })
        );
    }
}
