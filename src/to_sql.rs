use std::collections::HashSet;

use itertools::Itertools;

use crate::search::{
    ColorQuery, Comparison, ComparisonOperator, Name, PowerQuery, Search, SearchKeyword,
};

pub trait ToSql {
    fn to_sql(&self) -> String;
}

impl ToSql for PowerQuery {
    fn to_sql(&self) -> String {
        let operator = if self.is_negated {
            self.operator.negate()
        } else {
            self.operator.clone()
        };
        let clauses = match &self.comparison {
            Comparison::Number(num) => {
                format!("cards.power{operator}{num}", operator = operator, num = num)
            }
            Comparison::Tougness => {
                format!("cards.power{operator}cards.toughness", operator = operator)
            }
        };
        format!("({clauses})", clauses = clauses)
    }
}

impl ToSql for ColorQuery {
    fn to_sql(&self) -> String {
        let operator = if self.is_negated {
            self.operator.negate()
        } else {
            self.operator.clone()
        };
        let all_colors = ["W", "U", "B", "R", "G"].iter().map(|s| s.to_string());
        let all_colors_set: HashSet<String> = HashSet::from_iter(all_colors.clone());
        let colors = HashSet::from_iter(self.comparison.as_set());
        let clauses = match operator {
            ComparisonOperator::LessThan => {
                let positive = colors
                    .iter()
                    .sorted()
                    .map(|color| format!("cards.{color}=TRUE", color = color))
                    .join(" OR ");
                let not_all_positive = colors
                    .iter()
                    .sorted()
                    .map(|color| format!("cards.{color}=TRUE", color = color))
                    .join(" AND ");
                let not_all_positive = format!(
                    "NOT ({not_all_positive})",
                    not_all_positive = not_all_positive
                );
                let negative = all_colors_set
                    .difference(&colors)
                    .sorted()
                    .map(|c| format!("cards.{color}=FALSE", color = c))
                    .join(" AND ");
                format!(
                    "({positive}) AND ({not_all_positive}) AND ({negative})",
                    positive = positive,
                    not_all_positive = not_all_positive,
                    negative = negative
                )
            }
            ComparisonOperator::LessThanOrEqual => {
                let positive = colors
                    .iter()
                    .sorted()
                    .map(|color| format!("cards.{color}=TRUE", color = color))
                    .join(" OR ");
                let negative = all_colors_set
                    .difference(&colors)
                    .sorted()
                    .map(|c| format!("cards.{color}=FALSE", color = c))
                    .join(" AND ");
                format!(
                    "({positive}) AND ({negative})",
                    positive = positive,
                    negative = negative
                )
            }
            ComparisonOperator::NotEqual => colors
                .iter()
                .sorted()
                .map(|color| format!("cards.{color}=FALSE", color = color))
                .join(" AND "),
            ComparisonOperator::Equal => all_colors
                .sorted()
                .map(|color| {
                    if colors.contains(&color) {
                        format!("cards.{color}=TRUE", color = color)
                    } else {
                        format!("cards.{color}=FALSE", color = color)
                    }
                })
                .join(" AND "),
            ComparisonOperator::GreaterThan => {
                let at_least = colors
                    .iter()
                    .sorted()
                    .map(|color| format!("cards.{color}=TRUE", color = color))
                    .join(" AND ");
                let others = all_colors_set
                    .difference(&colors)
                    .sorted()
                    .map(|c| format!("cards.{color}=TRUE", color = c))
                    .join(" OR ");
                format!(
                    "({at_least}) AND ({others})",
                    at_least = at_least,
                    others = others
                )
            }
            ComparisonOperator::GreaterThanOrEqual => {
                let at_least = colors
                    .iter()
                    .sorted()
                    .map(|color| format!("cards.{color}=TRUE", color = color))
                    .join(" AND ");
                format!("{at_least}", at_least = at_least)
            }
        };
        format!("({clauses})", clauses = clauses)
    }
}

impl ToSql for Name {
    fn to_sql(&self) -> String {
        let like = format!("cards.name LIKE '%{name}%'", name = self.text);
        format!("({like})", like = like)
    }
}

impl ToSql for SearchKeyword {
    fn to_sql(&self) -> String {
        match self {
            SearchKeyword::Color(color) => color.to_sql(),
            SearchKeyword::Power(power) => power.to_sql(),
            SearchKeyword::Name(name) => name.to_sql(),
        }
    }
}

impl ToSql for Search {
    fn to_sql(&self) -> String {
        match self {
            Search::Keyword(keyword) => keyword.to_sql(),
            Search::And(queries) => queries
                .iter()
                .map(|query| query.to_sql())
                .collect::<Vec<_>>()
                .join(" AND "),
            Search::Or(queries) => queries
                .iter()
                .map(|query| query.to_sql())
                .collect::<Vec<_>>()
                .join(" OR "),
        }
    }
}

impl Search {
    pub fn to_clauses(&self) -> String {
        let clauses = self.to_sql();
        if !clauses.trim().is_empty() {
            format!(r"WHERE {}", clauses).trim().to_string()
        } else {
            clauses
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{self};

    #[test]
    pub fn equals_esper() {
        let search = search::search("c=ESPER").unwrap();
        let actual = search.to_sql();
        let expected =
            "(cards.B=TRUE AND cards.G=FALSE AND cards.R=FALSE AND cards.U=TRUE AND cards.W=TRUE)";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn not_equals_esper() {
        let search = search::search("c!=ESPER").unwrap();
        let actual = search.to_sql();
        let expected = "(cards.B=FALSE AND cards.U=FALSE AND cards.W=FALSE)";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn greater_or_equal_esper() {
        let search = search::search("c:ESPER").unwrap();
        let actual = search.to_sql();
        let expected = "(cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn other_greater_or_equal_esper() {
        let search = search::search("c>=ESPER").unwrap();
        let actual = search.to_sql();
        let expected = "(cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn less_than_or_equal_esper() {
        let search = search::search("c<=ESPER").unwrap();
        let actual = search.to_sql();
        let expected = "((cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (cards.G=FALSE AND cards.R=FALSE))";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn less_than_esper() {
        let search = search::search("c<ESPER").unwrap();
        let actual = search.to_sql();
        let expected = "((cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (NOT (cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)) AND (cards.G=FALSE AND cards.R=FALSE))";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn equals_esper_and_power_equals_touhgness() {
        let search = search::search("c=ESPER pow=toughness").unwrap();
        let actual = search.to_sql();
        let expected = "(cards.B=TRUE AND cards.G=FALSE AND cards.R=FALSE AND cards.U=TRUE AND cards.W=TRUE) AND (cards.power=cards.toughness)";
        assert_eq!(actual, expected)
    }
}
