use super::{color::ColorOperand, ColorOperator, ParsedSearch};

trait ToSearchString {
    fn to_search_string(&self) -> String;
}

impl ToSearchString for ParsedSearch {
    fn to_search_string(&self) -> String {
        todo!()
        // match self {
        //     ParsedSearch::Keyword(kw) => match kw {
        //         super::SearchKeyword::ColorQuery(ColorQuery {
        //             negated: is_negated,
        //             operator,
        //             operand: comparison,
        //         }) => {
        //             format!(
        //                 "{negated}color{operator}{comparison}",
        //                 negated = if *is_negated { "-" } else { "" },
        //                 operator = operator.to_search_string(),
        //                 comparison = comparison.to_search_string()
        //             )
        //         }
        //         super::SearchKeyword::PowerQuery(_) => todo!(),
        //         super::SearchKeyword::Name(_) => todo!(),
        //         super::SearchKeyword::TypeLineQuery(_) => todo!(),
        //     },
        //     ParsedSearch::And(_) => todo!(),
        //     ParsedSearch::Or(_) => todo!(),
        // }
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
