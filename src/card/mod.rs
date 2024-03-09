pub(crate) mod art_series;
pub(crate) mod card_data;
pub(crate) mod loading;
pub(crate) mod no_image;
pub(crate) mod normal;

use bytes::Bytes;
use iced::{
    alignment::{self},
    widget::{button, container},
    Command, Element,
};

use self::{
    art_series::ArtSeries, card_data::GetCardDataInfo, loading::LoadingCard, no_image::NoImageCard,
    normal::NormalCard,
};
use crate::{database::Database, Message, MessageError};

#[derive(Debug, Clone)]
pub enum Card {
    Normal(NormalCard),
    ArtSeries(ArtSeries),
    NoImage(NoImageCard),
    Loading(LoadingCard),
}

impl Card {
    pub fn load_action(&self) -> Command<Message> {
        if let Card::Loading(LoadingCard { id, .. }) = self {
            Command::perform(Card::get_card(id.to_string()), Message::CardLoaded)
        } else {
            Command::none()
        }
    }

    pub fn no_image_card(id: String, name: String, cmc: Option<f64>) -> Self {
        Self::NoImage(NoImageCard { id, name, cmc })
    }

    pub fn art_series(
        id: String,
        name: String,
        face: Option<Bytes>,
        selected_face: usize,
        num_faces: usize,
    ) -> Self {
        Self::ArtSeries(ArtSeries {
            id,
            name,
            face,
            selected_face,
            num_faces,
        })
    }

    pub fn loading(id: String) -> Self {
        Self::Loading(LoadingCard { id })
    }

    pub fn id(&self) -> String {
        match self {
            Card::Normal(normal) => normal.id().to_string(),
            Card::ArtSeries(art_series) => art_series.id.clone(),
            Card::Loading(LoadingCard { id, .. }) => id.clone(),
            Card::NoImage(no_image) => no_image.id.clone(),
        }
    }

    pub async fn get_card(id: String) -> Result<Card, MessageError> {
        let card_info = Database::get_card_data(id.clone())
            .await
            .expect("Couldn't get card info from db.");
        // .map_err(|_| MessageError::SQLQuery)?;
        if card_info.num_faces > 0 {
            let face = Database::get_card_face(card_info.id.clone(), 0)
                .await
                .expect("failed to get card from db");
            // .map_err(|_| MessageError::SQLQuery)?;
            return Ok(Card::art_series(
                card_info.id,
                card_info.name,
                face,
                0,
                card_info.num_faces,
            ));
        };
        let card = if card_info.has_image() {
            Card::Normal(NormalCard::from(card_info))
        } else {
            Card::no_image_card(card_info.id, card_info.name, card_info.cmc)
        };
        Ok(card)
    }

    pub async fn next_card_face(
        card_id: String,
        current_face: usize,
    ) -> Result<Card, MessageError> {
        let card_info = Database::get_card_data(card_id.clone())
            .await
            .map_err(|_| MessageError::SQLQuery)?;
        let next_face = if current_face + 1 < card_info.num_faces {
            current_face + 1
        } else {
            0
        };
        let face = Database::get_card_face(card_id.clone(), next_face)
            .await
            .expect("failed to get next card face from db");
        // .map_err(|_| MessageError::SQLQuery)?;
        Ok(Card::art_series(
            card_id,
            card_info.name,
            face,
            next_face,
            card_info.num_faces,
        ))
    }

    // TODO - I'd like to style the button to be transparent
    pub fn view(&self) -> Element<Message> {
        let height = 210 * 2;
        let width = 150 * 2;
        button(
            container(match self {
                Card::Normal(normal) => normal.view(),
                Card::ArtSeries(art_series) => art_series.view(),
                Card::Loading(loading_card) => loading_card.view(),
                Card::NoImage(no_image) => no_image.view(),
            })
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Top)
            .height(height)
            .width(width),
        )
        .on_press(Message::CardClicked { card_id: self.id() })
        .into()
    }
}
