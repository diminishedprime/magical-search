use iced::{widget::Column, Element};

use crate::{card::Card, Message, MessageError};

#[derive(Debug, Clone)]
pub enum CardDetail {
    Loaded { card: Card },
}

impl CardDetail {
    pub fn loaded(card: Card) -> Self {
        Self::Loaded { card }
    }
    pub async fn load_card_detail(card: Card) -> Result<CardDetail, MessageError> {
        Ok(CardDetail::loaded(card.clone()))
    }
    pub fn view(&self) -> Element<Message> {
        match self {
            CardDetail::Loaded { card } => Column::new().push(card.view()).into(),
        }
    }
}
