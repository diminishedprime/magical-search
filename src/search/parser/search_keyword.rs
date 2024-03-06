use nom::{branch::alt, sequence::delimited, IResult, Parser};
use nom_supreme::{error::ErrorTree, tag::complete::tag};

use super::{
    color_query::{color_query, ColorQuery},
    name,
    power_query::{power_query, PowerQuery},
    type_line_query::{type_line_query, TypeLineQuery},
    Name, ParsedSearch,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SearchKeyword {
    ColorQuery(ColorQuery),
    PowerQuery(PowerQuery),
    Name(Name),
    TypeLineQuery(TypeLineQuery),
}

pub fn search_keyword(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
    fn base_parser(input: &str) -> IResult<&str, ParsedSearch, ErrorTree<&str>> {
        alt((
            color_query.map(ParsedSearch::color_query),
            power_query.map(ParsedSearch::power_query),
            type_line_query.map(ParsedSearch::type_line),
            // Name must be the last parser since it's a bit of a catch-all.
            name.map(ParsedSearch::name),
        ))
        .parse(input)
    }

    alt((delimited(tag("("), base_parser, tag(")")), base_parser)).parse(input)
}
