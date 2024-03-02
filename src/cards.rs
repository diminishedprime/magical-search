use iced::{
    futures::{stream::FuturesOrdered, TryStreamExt},
    widget::{Column, Row},
    Element,
};
use itertools::Itertools;

use crate::{
    card::Card, database::Database, db::GET_CARDS_NAME_LIKE, search::Search, Message, MessageError,
    CARDS_PER_ROW, LIMIT,
};

#[derive(Debug, Clone)]
pub struct Cards {
    pub contents: Vec<Card>,
    pub cursor: usize,
}

impl Cards {
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

    pub async fn fetch_cards_with_query(search: Search) -> Result<Cards, MessageError> {
        let cards = Database::fetch_cards_with_search(search)
            .await
            .map_err(|_| MessageError::SQLQuery)?;
        Ok(cards)
    }

    pub async fn next_row(cursor: usize, search: Search) -> Result<Vec<String>, MessageError> {
        Ok(
            Database::fetch_card_ids(cursor, search)
                .await
                .expect("Unable to fetch card ids"), // .map_err(|_| MessageError::SQLQuery)?
        )
    }

    pub async fn fetch_cards_with_search(search: String) -> Result<Cards, MessageError> {
        let conn = Database::connection()
            .await
            .map_err(|_| MessageError::SQLConnection)?;

        let search = search.to_string();
        let cards = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(GET_CARDS_NAME_LIKE)?;
                let cards = stmt
                    .query_map(
                        &[
                            (":like", &format!("%{}%", search)),
                            (":limit", &LIMIT.to_string()),
                        ],
                        |row| {
                            let id: String = row.get(0)?;
                            Ok(id)
                        },
                    )?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(cards)
            })
            .await
            .expect("Coludn't load initial cards");
        // .map_err(|_| MessageError::SQLQuery)?;

        let cards: Vec<Card> = cards
            .into_iter()
            .map(|id| Card::get_card(id))
            .collect::<FuturesOrdered<_>>()
            .try_collect()
            .await?;

        Ok(Cards::new(cards))
    }
}
