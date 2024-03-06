use iced::Element;

use super::{ParsedSearch, Search};
use crate::Message;

impl Search {
    #[allow(dead_code)]
    pub fn view(&self, _depth: usize) -> Element<Message> {
        match &self.parsed_search {
            Some(parsed_search) => {
                match parsed_search {
                    ParsedSearch::Keyword(_) => todo!(),
                    ParsedSearch::And(_) => todo!(),
                    ParsedSearch::Or(_) => todo!(),
                    ParsedSearch::Negated(_, _) => todo!(),
                };
            }
            None => todo!(),
        }
    }
}
