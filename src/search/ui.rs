use iced::Element;

use super::Search;
use crate::Message;

impl Search {
    pub fn view(&self, depth: usize) -> Element<Message> {
        match &self.parsed_search {
            Some(parsed_search) => match parsed_search {
                super::ParsedSearch::Keyword(_) => todo!(),
                super::ParsedSearch::And(_) => todo!(),
                super::ParsedSearch::Or(_) => todo!(),
            },
            None => todo!(),
        }
    }
}
