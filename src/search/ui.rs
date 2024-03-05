use iced::Element;

use super::Search;
use crate::Message;

impl Search {
    #[allow(dead_code)]
    pub fn view(&self, _depth: usize) -> Element<Message> {
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
