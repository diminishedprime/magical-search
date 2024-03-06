use std::collections::HashSet;

use itertools::Itertools;

use crate::search::{
    and::And, or::Or, type_line_query::TypeLineQuery, ColorComparison, ColorQuery, Comparison,
    Name, ParsedSearch, PowerQuery, SearchKeyword,
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
        let all_colors = ["W", "U", "B", "R", "G"].iter().map(|s| s.to_string());
        let all_colors_set: HashSet<String> = HashSet::from_iter(all_colors.clone());
        let colors = HashSet::from_iter(self.comparison.as_set());
        let clauses = match self.operator {
            ColorComparison::LessThan => {
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
            // TODO - check this, not positive.
            ColorComparison::LessThanOrEqual | ColorComparison::Colon => {
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
            ColorComparison::NotEqual => colors
                .iter()
                .sorted()
                .map(|color| format!("cards.{color}=FALSE", color = color))
                .join(" AND "),
            ColorComparison::Equal => all_colors
                .sorted()
                .map(|color| {
                    if colors.contains(&color) {
                        format!("cards.{color}=TRUE", color = color)
                    } else {
                        format!("cards.{color}=FALSE", color = color)
                    }
                })
                .join(" AND "),
            ColorComparison::GreaterThan => {
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
            ColorComparison::GreaterThanOrEqual => {
                let at_least = colors
                    .iter()
                    .sorted()
                    .map(|color| format!("cards.{color}=TRUE", color = color))
                    .join(" AND ");
                format!("{at_least}", at_least = at_least)
            }
        };
        format!(
            "{negated}({clauses})",
            clauses = clauses,
            negated = if self.is_negated { " NOT " } else { "" }
        )
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
            SearchKeyword::TypeLine(type_line) => type_line.to_sql(),
        }
    }
}

impl ToSql for ParsedSearch {
    fn to_sql(&self) -> String {
        match self {
            ParsedSearch::Keyword(keyword) => keyword.to_sql(),
            ParsedSearch::And(And { items, negated }) | ParsedSearch::Or(Or { items, negated }) => {
                let queries = items
                    .iter()
                    .map(|query| query.to_sql())
                    .collect::<Vec<_>>()
                    .join(&format!(
                        " {} ",
                        if matches!(self, ParsedSearch::And(_)) {
                            "AND"
                        } else {
                            "OR"
                        }
                    ));
                if *negated {
                    format!("NOT ({queries})", queries = queries)
                } else {
                    queries
                }
            }
        }
    }
}

impl ToSql for TypeLineQuery {
    fn to_sql(&self) -> String {
        // TODO - I need to clean up this whole thing since this allows for the
        // potential of sql injection. Instead of returning String, I need to
        // return some sort of IntoClause trait or something similar that can
        // include the information on any parameters that need to be passed in.
        let clauses = format!("cards.type_line LIKE '%{}%'", self.comparison);
        format!("({clauses})", clauses = clauses)
    }
}

impl ParsedSearch {
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
    pub fn other_less_than_or_equal_esper() {
        let search = search::search("c:ESPER").unwrap();
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
