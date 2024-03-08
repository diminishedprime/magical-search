use super::{color::ColorOperand, ColorOperator, ColorQuery, ParsedSearch};

trait ToSearchString {
    fn to_search_string(&self) -> String;
}

impl ToSearchString for ParsedSearch {
    fn to_search_string(&self) -> String {
        match self {
            ParsedSearch::Or(or) => or
                .iter()
                .map(|parsed_search| parsed_search.to_search_string())
                .collect::<Vec<_>>()
                .join(" OR "),
            ParsedSearch::And(and) => and
                .iter()
                .map(|parsed_search| parsed_search.to_search_string())
                .collect::<Vec<_>>()
                .join(" AND "),
            ParsedSearch::Negated(negated, parsed_search) => {
                if *negated {
                    // TODO - I may want to handle this more cleanly
                    format!("-({})", parsed_search.to_search_string())
                } else {
                    parsed_search.to_search_string()
                }
            }
            ParsedSearch::Keyword(kw) => match kw {
                super::SearchKeyword::ColorQuery(ColorQuery { operator, operand }) => format!(
                    "color{operator}{operand}",
                    operator = operator.to_search_string(),
                    operand = operand.to_search_string()
                ),
                super::SearchKeyword::PowerQuery(_) => todo!(),
                super::SearchKeyword::Name(_) => todo!(),
                super::SearchKeyword::TypeLineQuery(_) => todo!(),
                super::SearchKeyword::Keyword(_) => todo!(),
                super::SearchKeyword::OracleQuery(_) => todo!(),
                super::SearchKeyword::ColorIdentityQuery(_) => todo!(),
            },
        }
    }
}

impl ToSearchString for ColorOperator {
    fn to_search_string(&self) -> String {
        match self {
            ColorOperator::LessThan => "<",
            ColorOperator::LessThanOrEqual => "<=",
            ColorOperator::NotEqual => "!=",
            ColorOperator::Colon => ":",
            ColorOperator::Equal => "=",
            ColorOperator::GreaterThan => ">",
            ColorOperator::GreaterThanOrEqual => ">=",
        }
        .to_string()
    }
}

impl ToSearchString for ColorOperand {
    fn to_search_string(&self) -> String {
        match self {
            ColorOperand::Red => "red",
            ColorOperand::Blue => "blue",
            ColorOperand::Black => "black",
            ColorOperand::Green => "green",
            ColorOperand::White => "white",
            ColorOperand::Azorius => "azorious",
            ColorOperand::Boros => "boros",
            ColorOperand::Dimir => "dimir",
            ColorOperand::Golgari => "golgari",
            ColorOperand::Gruul => "gruul",
            ColorOperand::Izzet => "izzet",
            ColorOperand::Orzhov => "orzhov",
            ColorOperand::Rakdos => "rakdos",
            ColorOperand::Selesnya => "selsnya",
            ColorOperand::Simic => "simic",
            ColorOperand::Colorless => "colorless",
            ColorOperand::Multicolor => "multicolor",
            ColorOperand::Abzan => "abzan",
            ColorOperand::Jeskai => "jeskai",
            ColorOperand::Sultai => "sultai",
            ColorOperand::Mardu => "mardu",
            ColorOperand::Temur => "temur",
            ColorOperand::Bant => "bant",
            ColorOperand::Esper => "esper",
            ColorOperand::Grixis => "grixis",
            ColorOperand::Jund => "jund",
            ColorOperand::Naya => "nay",
            ColorOperand::Aggression => "aggression",
            ColorOperand::Altruism => "altruism",
            ColorOperand::Growth => "growth",
            ColorOperand::Artifice => "artifice",
            ColorOperand::WUBRG => "wubrg",
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
