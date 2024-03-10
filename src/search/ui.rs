use iced::{
    widget::{column, row, text, Column, Rule, TextInput},
    Element, Length,
};

use super::{ParsedSearch, Search};
use crate::{Message, SPACING_MEDIUM, SPACING_SMALL};

impl Search {
    pub fn view_parsed_search(&self, depth: usize, item: &ParsedSearch) -> Element<Message> {
        match item {
            ParsedSearch::Keyword(kw) => match kw {
                super::SearchKeyword::ColorQuery(_) => todo!(),
                super::SearchKeyword::ColorIdentityQuery(ciq) => {
                    let operator = ciq.operator.describe();
                    let color = ciq.operand.describe();
                    text(format!("color identity {operator} {color}")).into()
                }
                super::SearchKeyword::PowerQuery(pq) => {
                    let operator = pq.operator.describe();
                    let operand = pq.operand.describe();
                    text(format!("power is {operator} {operand}")).into()
                }
                super::SearchKeyword::OracleQuery(oq) => {
                    let operand = &oq.oracle_text;
                    text(format!("oracle text contains {operand}")).into()
                }
                super::SearchKeyword::Name(_) => text("TODO: name controls").into(),
                super::SearchKeyword::TypeLineQuery(tlq) => {
                    let operand = &tlq.operand;
                    text(format!(r#"type includes: "{operand}""#)).into()
                }
                super::SearchKeyword::Keyword(_) => todo!(),
            },
            ParsedSearch::Or(items) | ParsedSearch::And(items) => {
                if items.len() == 1 {
                    return self.view_parsed_search(depth, &items[0]);
                }
                let and_or = if matches!(item, ParsedSearch::Or(_)) {
                    "or"
                } else {
                    "and"
                };

                let items: Vec<Element<Message>> = items
                    .iter()
                    .map(|item| self.view_parsed_search(depth + 1, item))
                    .collect();
                row!(
                    text(and_or),
                    Rule::vertical(1),
                    Column::with_children(items).spacing(SPACING_MEDIUM),
                )
                .align_items(iced::Alignment::Center)
                .spacing(SPACING_MEDIUM)
                .height(Length::Shrink)
                .into()
            }
            ParsedSearch::Negated(negated, item) => {
                if *negated {
                    row!(
                        text("not"),
                        Rule::vertical(1),
                        self.view_parsed_search(depth + 1, item),
                    )
                    .align_items(iced::Alignment::Center)
                    .spacing(SPACING_SMALL)
                    .height(Length::Shrink)
                    .into()
                } else {
                    self.view_parsed_search(depth, item)
                }
            }
        }
    }
    pub fn view(&self, _depth: usize) -> Element<Message> {
        let text_input = TextInput::new("Search", &self.input_text)
            .on_input(|input| Message::SearchInputChanged(input));
        let visual_search: Element<Message> = match &self.parsed_search {
            Some(parsed_search) => self.view_parsed_search(_depth, parsed_search),
            None => text("Current search is unparsable").into(),
        };
        column!(text_input, visual_search).into()
    }
}
