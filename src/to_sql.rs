use std::{collections::HashSet, sync::Mutex};

use itertools::Itertools;
use lazy_static::lazy_static;

use crate::search::{
    keyword::KeywordQuery, oracle_query::OracleQuery, type_line_query::TypeLineQuery,
    ColorOperator, ColorQuery, Name, ParsedSearch, PowerOperand, PowerOperator, PowerQuery,
    SearchKeyword,
};

lazy_static! {
    // Define a static Mutex-protected counter
    static ref COUNTER: Mutex<usize> = Mutex::new(0);
}

// Generate a unique table name
fn generate_table_name() -> String {
    let mut counter = COUNTER.lock().unwrap();
    let table_name = format!("t_{}", *counter);
    *counter += 1;
    table_name
}
pub struct SQL {
    where_clauses: String,
    join_clauses: Vec<String>,
}

impl Default for SQL {
    fn default() -> Self {
        SQL {
            where_clauses: "".to_string(),
            join_clauses: vec![],
        }
    }
}

impl SQL {
    pub fn new(where_clauses: String, join_clauses: Vec<String>) -> Self {
        SQL {
            where_clauses,
            join_clauses,
        }
    }

    pub fn joins(&self) -> String {
        self.join_clauses.join("\n")
    }

    pub fn wheres(&self) -> String {
        if self.where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", self.where_clauses)
        }
    }
}

pub trait ToSql {
    fn to_sql(&self) -> SQL;
}

impl ToSql for PowerQuery {
    fn to_sql(&self) -> SQL {
        let operator = match self.operator {
            PowerOperator::LessThan => "<",
            PowerOperator::LessThanOrEqual => "<=",
            PowerOperator::NotEqual => "!=",
            PowerOperator::Colon => "=",
            PowerOperator::Equal => "=",
            PowerOperator::GreaterThan => ">",
            PowerOperator::GreaterThanOrEqual => ">=",
        }
        .to_string();
        let clauses = match &self.operand {
            PowerOperand::Number(num) => {
                format!("cards.power{operator}{num}", operator = operator, num = num)
            }
            PowerOperand::Tougness => {
                format!("cards.power{operator}cards.toughness", operator = operator,)
            }
        };

        let _where = format!(
            "{negated}({clauses})",
            clauses = clauses,
            negated = if self.negated { " NOT " } else { "" }
        );
        SQL::new(_where, vec![])
    }
}

impl ToSql for ColorQuery {
    fn to_sql(&self) -> SQL {
        let all_colors = ["W", "U", "B", "R", "G"].iter().map(|s| s.to_string());
        let all_colors_set: HashSet<String> = HashSet::from_iter(all_colors.clone());
        let colors = HashSet::from_iter(self.operand.as_set());
        let clauses = match self.operator {
            ColorOperator::LessThan => {
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
            ColorOperator::LessThanOrEqual | ColorOperator::Colon => {
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
            ColorOperator::NotEqual => colors
                .iter()
                .sorted()
                .map(|color| format!("cards.{color}=FALSE", color = color))
                .join(" AND "),
            ColorOperator::Equal => all_colors
                .sorted()
                .map(|color| {
                    if colors.contains(&color) {
                        format!("cards.{color}=TRUE", color = color)
                    } else {
                        format!("cards.{color}=FALSE", color = color)
                    }
                })
                .join(" AND "),
            ColorOperator::GreaterThan => {
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
            ColorOperator::GreaterThanOrEqual => {
                let at_least = colors
                    .iter()
                    .sorted()
                    .map(|color| format!("cards.{color}=TRUE", color = color))
                    .join(" AND ");
                format!("{at_least}", at_least = at_least)
            }
        };
        let _where = format!("{clauses}", clauses = clauses);
        SQL::new(_where, vec![])
    }
}

impl ToSql for Name {
    fn to_sql(&self) -> SQL {
        let like = format!("cards.name LIKE '%{name}%'", name = self.text);
        let _where = format!("({like})", like = like);
        SQL::new(_where, vec![])
    }
}

impl ToSql for SearchKeyword {
    fn to_sql(&self) -> SQL {
        match self {
            SearchKeyword::ColorQuery(color) => color.to_sql(),
            SearchKeyword::PowerQuery(power) => power.to_sql(),
            SearchKeyword::Name(name) => name.to_sql(),
            SearchKeyword::TypeLineQuery(type_line) => type_line.to_sql(),
            SearchKeyword::Keyword(kw) => kw.to_sql(),
            SearchKeyword::OracleQuery(oq) => oq.to_sql(),
        }
    }
}

impl ToSql for OracleQuery {
    fn to_sql(&self) -> SQL {
        let like = format!(
            "cards.oracle_text LIKE '%{oracle_text}%'",
            oracle_text = self.oracle_text
        );
        let _where = format!("({like})", like = like);
        SQL::new(_where, vec![])
    }
}

impl ToSql for KeywordQuery {
    fn to_sql(&self) -> SQL {
        let alias = generate_table_name();
        let _join = vec![format!(
            "LEFT JOIN card_keywords {alias} ON cards.id = {alias}.card_id",
            alias = alias,
        )];
        let _where = format!(
            "{alias}.keyword LIKE '%{keyword}%'",
            keyword = self.keyword,
            alias = alias
        );
        SQL::new(_where, _join)
    }
}

impl ToSql for ParsedSearch {
    fn to_sql(&self) -> SQL {
        match self {
            ParsedSearch::Keyword(keyword) => keyword.to_sql(),
            ParsedSearch::And(operands) | ParsedSearch::Or(operands) => {
                let sqls = operands
                    .iter()
                    .map(|query| query.to_sql())
                    .collect::<Vec<_>>();
                let _where = sqls
                    .iter()
                    .map(|sql| sql.where_clauses.clone())
                    .collect::<Vec<_>>()
                    .join(&format!(
                        " {} ",
                        if matches!(self, ParsedSearch::And(_)) {
                            "AND"
                        } else {
                            "OR"
                        }
                    ));
                let _join = sqls
                    .iter()
                    .map(|sql| sql.join_clauses.clone())
                    .flatten()
                    .collect();
                SQL::new(_where, _join)
            }
            ParsedSearch::Negated(negated, search) => {
                let sql = search.to_sql();
                let _where = format!(
                    "{negated}({search})",
                    negated = if *negated { "NOT " } else { "" },
                    search = sql.where_clauses
                );
                SQL::new(_where, sql.join_clauses)
            }
        }
    }
}

impl ToSql for TypeLineQuery {
    fn to_sql(&self) -> SQL {
        // TODO - I need to clean up this whole thing since this allows for the
        // potential of sql injection. Instead of returning String, I need to
        // return some sort of IntoClause trait or something similar that can
        // include the information on any parameters that need to be passed in.
        let clauses = format!("cards.type_line LIKE '%{}%'", self.operand);
        let _where = format!("({clauses})", clauses = clauses);
        SQL::new(_where, vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{self};

    #[test]
    pub fn equals_esper() {
        let search = search::search("c=ESPER").unwrap();
        let actual = search.to_sql().where_clauses;
        let expected =
            "(cards.B=TRUE AND cards.G=FALSE AND cards.R=FALSE AND cards.U=TRUE AND cards.W=TRUE)";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn not_equals_esper() {
        let search = search::search("c!=ESPER").unwrap();
        let actual = search.to_sql().where_clauses;
        let expected = "(cards.B=FALSE AND cards.U=FALSE AND cards.W=FALSE)";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn not_esper_and_not_golgari() {
        let search = search::search("c!=ESPER c!=GOLGARI").unwrap();
        let actual = search.to_sql().where_clauses;
        let expected = "(cards.B=FALSE AND cards.U=FALSE AND cards.W=FALSE) AND (cards.B=FALSE AND cards.G=FALSE)";
        assert_eq!(actual, expected);
    }

    #[test]
    pub fn other_greater_or_equal_esper() {
        let search = search::search("c>=ESPER").unwrap();
        let actual = search.to_sql().where_clauses;
        let expected = "(cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn less_than_or_equal_esper() {
        let search = search::search("c<=ESPER").unwrap();
        let actual = search.to_sql().where_clauses;
        let expected = "((cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (cards.G=FALSE AND cards.R=FALSE))";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn other_less_than_or_equal_esper() {
        let search = search::search("c:ESPER").unwrap();
        let actual = search.to_sql().where_clauses;
        let expected =
            "((cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (cards.G=FALSE AND cards.R=FALSE))";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn less_than_esper() {
        let search = search::search("c<ESPER").unwrap();
        let actual = search.to_sql().where_clauses;
        let expected = "((cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (NOT (cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)) AND (cards.G=FALSE AND cards.R=FALSE))";
        assert_eq!(actual, expected)
    }

    #[test]
    pub fn equals_esper_and_power_equals_touhgness() {
        let search = search::search("c=ESPER pow=toughness").unwrap();
        let actual = search.to_sql().where_clauses;
        let expected = "(cards.B=TRUE AND cards.G=FALSE AND cards.R=FALSE AND cards.U=TRUE AND cards.W=TRUE) AND ((cards.power=cards.toughness))";
        assert_eq!(actual, expected)
    }
}
