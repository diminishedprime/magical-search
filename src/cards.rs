use crate::{
    card::Card,
    db::{GET_CARDS, GET_CARDS_NAME_LIKE},
    MagicalSearch, Message, MessageError, LIMIT,
};
use iced::{
    widget::{Column, Row},
    Element,
};
use itertools::Itertools;
use tokio_rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct Cards(pub Vec<Card>);

impl Cards {
    fn new(cards: Vec<Card>) -> Self {
        Self(cards)
    }

    pub fn view(&self) -> Element<Message> {
        let mut image_grid = Column::new();
        let chunks = (LIMIT as f64).sqrt().ceil() as usize;
        for row in &self.0.iter().chunks(chunks) {
            let mut row_container = Row::new();
            for card in row {
                row_container = row_container.push(card.view());
            }
            image_grid = image_grid.push(row_container);
        }
        image_grid.into()
    }

    pub async fn fetch_cards() -> Result<Cards, MessageError> {
        let path = MagicalSearch::db_path();
        let conn = Connection::open(path)
            .await
            .map_err(|_| MessageError::SQLConnection)?;

        let cards = conn
            .call(|conn| {
                let mut stmt = conn.prepare(GET_CARDS)?;
                let cards = stmt
                    .query_map(&[(":limit", &LIMIT.to_string())], |row| {
                        let id: String = row.get(0)?;
                        let name: String = row.get(1)?;
                        let cmc: Option<f64> = row.get(2)?;
                        let url: Option<String> = row.get(3)?;
                        let image: Option<Vec<u8>> = row.get(4)?;
                        if image.is_none() {
                            Ok(Card::loading(id, name, cmc, url))
                        } else {
                            Ok(Card::fully_loaded(id, name, cmc, image))
                        }
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(cards)
            })
            .await
            .map_err(|_| MessageError::SQLQuery)?;

        Ok(Cards::new(cards))
    }

    pub async fn fetch_cards_with_search(search: String) -> Result<Cards, MessageError> {
        let path = MagicalSearch::db_path();
        let conn = Connection::open(path)
            .await
            .map_err(|_| MessageError::SQLConnection)?;

        let search = search.to_string();
        let cards = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(GET_CARDS_NAME_LIKE)?;
                let cards = stmt
                    .query_map(&[(":like", &format!("%{}%", search)), (":limit", &LIMIT.to_string())], |row| {
                        let id: String = row.get(0)?;
                        let name: String = row.get(1)?;
                        let cmc: Option<f64> = row.get(2)?;
                        let url: Option<String> = row.get(3)?;
                        let image: Option<Vec<u8>> = row.get(4)?;
                        if image.is_none() {
                            Ok(Card::loading(id, name, cmc, url))
                        } else {
                            Ok(Card::fully_loaded(id, name, cmc, image))
                        }
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(cards)
            })
            .await
            .map_err(|_| MessageError::SQLQuery)?;

        Ok(Cards::new(cards))
    }
}
