use std::{collections::HashSet, iter::once, sync::Mutex};

use itertools::Itertools;
use lazy_static::lazy_static;

use crate::search::{
    color::ColorOperand,
    color_identity_query::{ColorIdentityOperator, ColorIdentityQuery},
    keyword::KeywordQuery,
    oracle_query::OracleQuery,
    type_line_query::TypeLineQuery,
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

#[derive(Debug)]
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

impl ColorIdentityQuery {
    fn colors_true(&self) -> Vec<String> {
        self.operand
            .as_set()
            .iter()
            .sorted()
            .map(|color| format!("cards.{color}=TRUE", color = color))
            .collect()
    }
    fn colors_true_or(&self) -> String {
        self.colors_true().iter().join(" OR ")
    }
    fn colors_true_and(&self) -> String {
        self.colors_true().iter().join(" AND ")
    }
    fn not_my_colors_true_and(&self) -> String {
        let my_colors_true_and = self.colors_true_and();
        if my_colors_true_and.is_empty() {
            "".to_string()
        } else {
            format!("NOT ({my_colors_true_and})")
        }
    }
    // fn colors_false(&self) -> Vec<String> {
    //     self.operand
    //         .as_set()
    //         .iter()
    //         .sorted()
    //         .map(|color| format!("cards.{color}=FALSE", color = color))
    //         .collect()
    // }
    fn difference_true(&self) -> Vec<String> {
        let my_colors = self.operand.as_set();
        let all_colors_set = ColorOperand::all_colors_set();
        all_colors_set
            .difference(&my_colors)
            .sorted()
            .map(|c| format!("cards.{color}=TRUE", color = c))
            .collect()
    }
    fn other_colors_true_and(&self) -> String {
        self.difference_true().iter().join(" AND ")
    }
    fn other_colors_true_or(&self) -> String {
        self.difference_true().iter().join(" OR ")
    }
    fn other_colors_false_and(&self) -> String {
        self.difference_false().iter().join(" AND ")
    }
    fn not_other_colors_true_and(&self) -> String {
        let other_colors_true_and = self.other_colors_true_and();
        if other_colors_true_and.is_empty() {
            "".to_string()
        } else {
            format!("NOT ({other_colors_true_and})")
        }
    }
    fn not_other_colors_true_or(&self) -> String {
        let other_colors_true_and = self.other_colors_true_or();
        if other_colors_true_and.is_empty() {
            "".to_string()
        } else {
            format!("NOT ({other_colors_true_and})")
        }
    }
    fn difference_false(&self) -> Vec<String> {
        let my_colors = self.operand.as_set();
        let all_colors_set = ColorOperand::all_colors_set();
        all_colors_set
            .difference(&my_colors)
            .sorted()
            .map(|c| format!("cards.{color}=FALSE", color = c))
            .collect()
    }
}

impl ColorIdentityQuery {
    // Less than a color identity for the example of esper means the card can
    // have White OR Blue OR Black, but not all of them. i.e.
    // (W OR U OR B) and NOT (W AND U AND B) and NOT (G OR R)
    fn color_identity_less_than(&self) -> String {
        match (
            self.colors_true_or().as_str(),
            self.not_my_colors_true_and().as_str(),
            self.not_other_colors_true_and().as_str(),
        ) {
            (only, "", "") | ("", only, "") | ("", "", only) => format!("{only}"),
            (first, second, "") | ("", first, second) | (first, "", second) => {
                format!("({first}) AND ({second})")
            }
            (first, second, third) => format!("({first}) AND ({second}) AND ({third})"),
        }
    }

    // Less than or equal a color identity means for the example of esper that
    // we have at least the esper colors and none of the other ones. i.e.
    // (W OR U OR B) AND (NOT G AND NOT R)
    fn color_identity_less_than_or_equal(&self) -> String {
        match (
            self.colors_true_or().as_str(),
            self.other_colors_false_and().as_str(),
        ) {
            ("", only) | (only, "") => format!("{only}"),
            (first, second) => format!("({}) AND ({})", first, second),
        }
    }

    // Not equal to a color identity for the example of esper means that the
    // identity isn't exactly esper i.e.
    // NOT (W AND U AND B)
    fn color_identity_not_equal(&self) -> String {
        self.not_my_colors_true_and()
    }

    // Equal to the color identity for the example of esper means that
    // the card must have exactly the colors of esper i.e.
    // W AND U AND B
    fn color_identity_equal(&self) -> String {
        once(self.colors_true_and())
            .chain(once(self.not_other_colors_true_or()))
            .filter(|s| !s.is_empty())
            .join(" AND ")
    }

    // Greater than a color identity for the example of esper means
    // that the identity must have 1 more color than esper in
    // addition to the esper colors. i.e.
    // W AND U AND B AND (G OR R)
    fn color_identity_greater_than(&self) -> String {
        // TODO - This technically doesn't work correctly with WUBRG, but I
        // don't have error handling so I'm just letting this not work
        // correctly for WUBRG.
        match (
            self.colors_true_and().as_str(),
            self.other_colors_true_or().as_str(),
        ) {
            ("", only) | (only, "") => format!("{only}"),
            (first, second) => format!("({}) AND ({})", first, second),
        }
    }

    fn color_identity_greater_than_or_equal(&self) -> String {
        self.colors_true_and()
    }
}

impl ToSql for ColorIdentityQuery {
    fn to_sql(&self) -> SQL {
        let clauses = match self.operator {
            ColorIdentityOperator::LessThan => self.color_identity_less_than(),
            ColorIdentityOperator::LessThanOrEqual | ColorIdentityOperator::Colon => {
                self.color_identity_less_than_or_equal()
            }
            ColorIdentityOperator::NotEqual => self.color_identity_not_equal(),
            ColorIdentityOperator::Equal => self.color_identity_equal(),
            ColorIdentityOperator::GreaterThan => self.color_identity_greater_than(),
            ColorIdentityOperator::GreaterThanOrEqual => {
                self.color_identity_greater_than_or_equal()
            }
        };
        let _where = format!("{clauses}", clauses = clauses);
        SQL::new(_where, vec![])
    }
}

impl ToSql for ColorQuery {
    fn to_sql(&self) -> SQL {
        let all_colors = ColorOperand::all_colors().into_iter();
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
            ColorOperator::LessThanOrEqual => {
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
            ColorOperator::GreaterThanOrEqual | ColorOperator::Colon => {
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
            SearchKeyword::ColorIdentityQuery(ciq) => ciq.to_sql(),
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
                if operands.len() == 1 {
                    return operands[0].to_sql();
                }
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
                let _where = if *negated {
                    format!(" NOT ({search})", search = sql.where_clauses)
                } else {
                    sql.where_clauses
                };
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
        if self.operand.is_empty() {
            SQL::default()
        } else {
            let clauses = format!("cards.type_line LIKE '%{}%'", self.operand);
            let _where = format!("({clauses})", clauses = clauses);
            SQL::new(_where, vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{self};

    // #[test]
    // pub fn equals_esper() {
    //     let search = search::search("c=ESPER").unwrap();
    //     let actual = search.to_sql().where_clauses;
    //     let expected =
    //         "(cards.B=TRUE AND cards.G=FALSE AND cards.R=FALSE AND cards.U=TRUE AND cards.W=TRUE)";
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn not_equals_esper() {
    //     let search = search::search("c!=ESPER").unwrap();
    //     let actual = search.to_sql().where_clauses;
    //     let expected = "(cards.B=FALSE AND cards.U=FALSE AND cards.W=FALSE)";
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn not_esper_and_not_golgari() {
    //     let search = search::search("c!=ESPER c!=GOLGARI").unwrap();
    //     let actual = search.to_sql().where_clauses;
    //     let expected = "(cards.B=FALSE AND cards.U=FALSE AND cards.W=FALSE) AND (cards.B=FALSE AND cards.G=FALSE)";
    //     assert_eq!(actual, expected);
    // }

    // #[test]
    // pub fn other_greater_or_equal_esper() {
    //     let search = search::search("c>=ESPER").unwrap();
    //     let actual = search.to_sql().where_clauses;
    //     let expected = "(cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)";
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn less_than_or_equal_esper() {
    //     let search = search::search("c<=ESPER").unwrap();
    //     let actual = search.to_sql().where_clauses;
    //     let expected = "((cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (cards.G=FALSE AND cards.R=FALSE))";
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn other_less_than_or_equal_esper() {
    //     let search = search::search("c:ESPER").unwrap();
    //     let actual = search.to_sql().where_clauses;
    //     let expected = "(cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)";
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn less_than_esper() {
    //     let search = search::search("c<ESPER").unwrap();
    //     let actual = search.to_sql().where_clauses;
    //     let expected = "((cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (NOT (cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)) AND (cards.G=FALSE AND cards.R=FALSE))";
    //     assert_eq!(actual, expected)
    // }

    // #[test]
    // pub fn equals_esper_and_power_equals_touhgness() {
    //     let search = search::search("c=ESPER pow=toughness").unwrap();
    //     let actual = search.to_sql().where_clauses;
    //     let expected = "(cards.B=TRUE AND cards.G=FALSE AND cards.R=FALSE AND cards.U=TRUE AND cards.W=TRUE) AND ((cards.power=cards.toughness))";
    //     assert_eq!(actual, expected)
    // }

    #[test]
    fn id_esper_less_than() {
        let actual = search::search("id<esper").unwrap().to_sql().where_clauses;
        let expected = "(cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (NOT (cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)) AND (NOT (cards.G=TRUE AND cards.R=TRUE))";
        assert_eq!(actual, expected);
    }
    #[test]
    fn id_esper_less_than_or_equal() {
        let actual = search::search("id<=esper").unwrap().to_sql().where_clauses;
        let expected =
            "(cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (cards.G=FALSE AND cards.R=FALSE)";
        assert_eq!(actual, expected);
    }
    #[test]
    fn id_esper_not_equal() {
        let actual = search::search("id!=esper").unwrap().to_sql().where_clauses;
        let expected = "NOT (cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE)";
        assert_eq!(actual, expected);
    }
    #[test]
    fn id_esper_colon() {
        let actual = search::search("id:esper").unwrap().to_sql().where_clauses;
        let expected =
            "(cards.B=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (cards.G=FALSE AND cards.R=FALSE)";
        assert_eq!(actual, expected);
    }
    #[test]
    fn id_esper_equal() {
        let actual = search::search("id=esper").unwrap().to_sql().where_clauses;
        let expected =
            "cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE AND NOT (cards.G=TRUE OR cards.R=TRUE)";
        assert_eq!(actual, expected);
    }
    #[test]
    fn id_esper_greater_than() {
        let actual = search::search("id>esper").unwrap().to_sql().where_clauses;
        let expected =
            "(cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE) AND (cards.G=TRUE OR cards.R=TRUE)";
        assert_eq!(actual, expected);
    }
    #[test]
    fn id_esper_greater_than_or_equal() {
        let actual = search::search("id>=esper").unwrap().to_sql().where_clauses;
        let expected = "cards.B=TRUE AND cards.U=TRUE AND cards.W=TRUE";
        assert_eq!(actual, expected);
    }

    #[test]
    fn id_wubrg_less_than() {
        let actual = search::search("id<wubrg").unwrap().to_sql().where_clauses;
        let expected = "(cards.B=TRUE OR cards.G=TRUE OR cards.R=TRUE OR cards.U=TRUE OR cards.W=TRUE) AND (NOT (cards.B=TRUE AND cards.G=TRUE AND cards.R=TRUE AND cards.U=TRUE AND cards.W=TRUE))";
        assert_eq!(actual, expected);
    }

    #[test]
    fn id_wubrg_less_than_or_equal() {
        let actual = search::search("id<=wubrg").unwrap().to_sql().where_clauses;
        let expected =
            "cards.B=TRUE OR cards.G=TRUE OR cards.R=TRUE OR cards.U=TRUE OR cards.W=TRUE";
        assert_eq!(actual, expected);
    }

    #[test]
    fn id_wubrg_not_equal() {
        let actual = search::search("id!=wubrg").unwrap().to_sql().where_clauses;
        let expected = "NOT (cards.B=TRUE AND cards.G=TRUE AND cards.R=TRUE AND cards.U=TRUE AND cards.W=TRUE)";
        assert_eq!(actual, expected);
    }

    #[test]
    fn id_wubrg_colon() {
        let actual = search::search("id:wubrg").unwrap().to_sql().where_clauses;
        let expected =
            "cards.B=TRUE OR cards.G=TRUE OR cards.R=TRUE OR cards.U=TRUE OR cards.W=TRUE";
        assert_eq!(actual, expected);
    }

    #[test]
    fn id_wubrg_equals() {
        let actual = search::search("id=wubrg").unwrap().to_sql().where_clauses;
        let expected =
            "cards.B=TRUE AND cards.G=TRUE AND cards.R=TRUE AND cards.U=TRUE AND cards.W=TRUE";
        assert_eq!(actual, expected);
    }

    #[test]
    fn id_wubrg_greater_than() {
        let actual = search::search("id>wubrg").unwrap().to_sql().where_clauses;
        // This isn't really right, but it'll do for now. I kinda doubt magic
        // will add more colors.
        let expected =
            "cards.B=TRUE AND cards.G=TRUE AND cards.R=TRUE AND cards.U=TRUE AND cards.W=TRUE";
        assert_eq!(actual, expected);
    }

    #[test]
    fn id_wubrg_greater_than_or_equal() {
        let actual = search::search("id>=wubrg").unwrap().to_sql().where_clauses;
        // This isn't really right, but it'll do for now. I kinda doubt magic
        // will add more colors.
        let expected =
            "cards.B=TRUE AND cards.G=TRUE AND cards.R=TRUE AND cards.U=TRUE AND cards.W=TRUE";
        assert_eq!(actual, expected);
    }

    #[test]
    fn id_rakdos_equal() {
        let actual = search::search("id=rakdos").unwrap().to_sql().where_clauses;
        let expected =
            "cards.B=TRUE AND cards.R=TRUE AND NOT (cards.G=TRUE OR cards.U=TRUE OR cards.W=TRUE)";
        assert_eq!(actual, expected);
    }
}
