use bytes::Bytes;
use iced::{
    widget::{column, image::Handle, row, text, Image},
    Element,
};

use super::card_data::{CardData, GetCardDataInfo};
use crate::Message;

#[derive(Debug, Clone)]
pub struct NormalCard {
    card_data: CardData,
}

impl GetCardDataInfo for NormalCard {
    fn card_data<'a>(&'a self) -> &'a CardData {
        &self.card_data
    }
}

impl From<CardData> for NormalCard {
    fn from(card_data: CardData) -> Self {
        Self { card_data }
    }
}

impl NormalCard {
    fn image(&self) -> Option<Bytes> {
        self.card_data.image.as_ref().map(|a| a.clone())
    }

    pub fn view(&self) -> Element<Message> {
        if let Some(image) = &self.image() {
            column!(Image::new(Handle::from_memory(image.clone()))
                .content_fit(iced::ContentFit::Contain),)
        } else {
            column!(
                row!(text("Name:"), text(&self.name()),),
                row!(
                    text("cmc:"),
                    text(
                        self.cmc()
                            .map(|cmc| cmc.to_string())
                            .unwrap_or("".to_string())
                    )
                ),
                text(&self.oracle_text().unwrap_or(""))
            )
            .align_items(iced::Alignment::Start)
        }
        .into()
    }
}
