use nom::{branch::alt, sequence::tuple, IResult, Parser};
use nom_supreme::{error::ErrorTree, tag::complete::tag_no_case};

use super::{name::quoted_or_until_space, parsed_search::SearchKeyword, ParsedSearch};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleQuery {
    pub oracle_text: String,
}

pub fn oracle_query(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    tuple((
        alt((tag_no_case("o:"), tag_no_case("oracle:"))),
        quoted_or_until_space,
    ))
    .map(|(_, oracle_text)| OracleQuery {
        oracle_text: oracle_text.to_string(),
    })
    .map(ParsedSearch::oracle_query)
    .parse(input)
}

impl ParsedSearch {
    pub fn oracle_query(oracle_query: OracleQuery) -> Self {
        Self::Keyword(SearchKeyword::OracleQuery(oracle_query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_text_includes_doublestrike() {
        let (_, actual) = oracle_query(r#"o:"Double Strike""#).unwrap();
        assert_eq!(
            actual,
            ParsedSearch::oracle_query(OracleQuery {
                oracle_text: "Double Strike".to_string()
            })
        );
    }
}
