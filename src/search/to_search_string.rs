use super::{color::Color, ColorQuery, ParsedSearch, Search};

trait ToSearchString {
    fn to_search_string(&self) -> String;
}

impl ToSearchString for ParsedSearch {
    fn to_search_string(&self) -> String {
        match self {
            ParsedSearch::Keyword(kw) => match kw {
                super::SearchKeyword::Color(ColorQuery {
                    is_negated,
                    operator,
                    comparison,
                }) => {
                    format!(
                        "{negated}color{operator}{comparison}",
                        negated = if *is_negated { "-" } else { "" },
                        operator = match operator {
                            crate::search::ComparisonOperator::LessThan => "<",
                            crate::search::ComparisonOperator::LessThanOrEqual => "<=",
                            crate::search::ComparisonOperator::NotEqual => "!=",
                            crate::search::ComparisonOperator::Equal => "=",
                            crate::search::ComparisonOperator::GreaterThan => ">",
                            crate::search::ComparisonOperator::GreaterThanOrEqual => ">=",
                        },
                        comparison = comparison.to_search_string()
                    )
                }
                super::SearchKeyword::Power(_) => todo!(),
                super::SearchKeyword::Name(_) => todo!(),
                super::SearchKeyword::TypeLine(_) => todo!(),
            },
            ParsedSearch::And(_) => todo!(),
            ParsedSearch::Or(_) => todo!(),
        }
    }
}

impl ToSearchString for Color {
    fn to_search_string(&self) -> String {
        match self {
            Color::Red => "red",
            Color::Blue => "blue",
            Color::Black => "black",
            Color::Green => "green",
            Color::White => "white",
            Color::Azorius => "azorious",
            Color::Boros => "boros",
            Color::Dimir => "dimir",
            Color::Golgari => "golgari",
            Color::Gruul => "gruul",
            Color::Izzet => "izzet",
            Color::Orzhov => "orzhov",
            Color::Rakdos => "rakdos",
            Color::Selesnya => "selsnya",
            Color::Simic => "simic",
            Color::Colorless => "colorless",
            Color::Multicolor => "multicolor",
            Color::Abzan => "abzan",
            Color::Jeskai => "jeskai",
            Color::Sultai => "sultai",
            Color::Mardu => "mardu",
            Color::Temur => "temur",
            Color::Bant => "bant",
            Color::Esper => "esper",
            Color::Grixis => "grixis",
            Color::Jund => "jund",
            Color::Naya => "nay",
            Color::Aggression => "aggression",
            Color::Altruism => "altruism",
            Color::Growth => "growth",
            Color::Artifice => "artifice",
            Color::WUBRG => "wubrg",
        }
        .to_string()
    }
}

#[cfg(test)]
mod test {
    use super::ToSearchString;
    use crate::search::search;

    #[test]
    fn color_keyword_happy_path() {
        let expected = "color=esper";
        let parsed = search(expected).unwrap();
        let actual = parsed.to_search_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn color_upgrade_keyword_happy_path() {
        let expected = "color=esper";
        let input = "c=esper";
        let parsed = search(input).unwrap();
        let actual = parsed.to_search_string();
        assert_eq!(expected, actual);
    }
}
