use iced::{
    widget::{Column, Row},
    Command, Element,
};
use itertools::Itertools;

use crate::{card::Card, database::Database, search::Search, Message, MessageError, CARDS_PER_ROW};

#[derive(Debug, Clone)]
pub struct Cards {
    pub contents: Vec<Card>,
    pub cursor: usize,
}

impl Cards {
    pub fn clear(&mut self) {
        self.contents.clear();
        self.cursor = 0;
    }
    pub fn new(cards: Vec<Card>) -> Self {
        Self {
            cursor: cards.len(),
            contents: cards,
        }
    }

    pub fn extend_cards<I: IntoIterator<Item = Card>>(&mut self, iter: I) {
        self.contents.extend(iter);
        self.cursor = self.contents.len();
    }

    pub fn view(&self) -> Element<Message> {
        let mut image_grid = Column::new();
        for row in &self.contents.iter().chunks(CARDS_PER_ROW) {
            let mut row_container = Row::new();
            for card in row {
                row_container = row_container.push(card.view());
            }
            image_grid = image_grid.push(row_container);
        }
        image_grid.into()
    }

    pub async fn next_row(cursor: usize, search: Search) -> Result<Vec<String>, MessageError> {
        Ok(
            Database::fetch_card_ids(cursor, search)
                .await
                .expect("Unable to fetch card ids"), // .map_err(|_| MessageError::SQLQuery)?
        )
    }

    pub fn initial_rows_for(search: Search) -> Command<Message> {
        Command::perform(Cards::next_row(0, search.clone()), Message::LoadRow)
    }
}
