use iced::{
    futures::{stream::FuturesOrdered, TryStreamExt},
    widget::{Column, Row},
    Element,
};
use itertools::Itertools;
use tokio_rusqlite::Connection;

use crate::{
    card::Card, db::GET_CARDS_NAME_LIKE, MagicalSearch, Message, MessageError, CARDS_PER_ROW, LIMIT,
};

#[derive(Debug, Clone)]
pub struct Cards(pub Vec<Card>);

impl Cards {
    fn new(cards: Vec<Card>) -> Self {
        Self(cards)
    }

    pub fn view(&self) -> Element<Message> {
        let mut image_grid = Column::new();
        for row in &self.0.iter().chunks(CARDS_PER_ROW) {
            let mut row_container = Row::new();
            for card in row {
                row_container = row_container.push(card.view());
            }
            image_grid = image_grid.push(row_container);
        }
        image_grid.into()
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
