use super::{
    color_query::ColorQuery, keyword::KeywordQuery, power_query::PowerQuery,
    type_line_query::TypeLineQuery, Name,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SearchKeyword {
    ColorQuery(ColorQuery),
    PowerQuery(PowerQuery),
    Name(Name),
    TypeLineQuery(TypeLineQuery),
    Keyword(KeywordQuery),
}
