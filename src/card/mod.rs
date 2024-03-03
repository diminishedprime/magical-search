pub(crate) mod art_series;
pub(crate) mod card_data;
pub(crate) mod loading;
pub(crate) mod no_image;
pub(crate) mod normal;

use iced::{
    alignment::{self},
    widget::{button, container},
    Command, Element,
};
use rusqlite::named_params;
use tokio::spawn;

use self::{
    art_series::ArtSeries,
    card_data::{CardData, ImageInfo, ImageSize},
    loading::LoadingCard,
    no_image::NoImageCard,
    normal::NormalCard,
};
use crate::{
    database::Database,
    db::{GET_CARD, GET_CARD_FACE, WRITE_FACE_SMALL_BLOB},
    Message, MessageError,
};

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

    pub fn normal_card(id: String, name: String, cmc: Option<f64>, image: Vec<u8>) -> Self {
        Self::Normal(NormalCard {
            id,
            name,
            cmc,
            image,
        })
    }

    pub fn no_image_card(id: String, name: String, cmc: Option<f64>) -> Self {
        Self::NoImage(NoImageCard { id, name, cmc })
    }

    pub fn art_series(
        id: String,
        name: String,
        face: Option<Vec<u8>>,
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
            Card::Normal(normal) => normal.id.clone(),
            Card::ArtSeries(art_series) => art_series.id.clone(),
            Card::Loading(LoadingCard { id, .. }) => id.clone(),
            Card::NoImage(no_image) => no_image.id.clone(),
        }
    }

    pub async fn get_card_info(id: String) -> Result<CardData, MessageError> {
        let conn = Database::connection()
            .await
            .map_err(|_| MessageError::SQLConnection)?;

        let id = id.to_string();
        conn.call(move |conn| {
            let mut stmt = conn.prepare(GET_CARD)?;
            let card = stmt.query_row(&[(":id", &id)], |row| {
                Ok(CardData {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cmc: row.get(2)?,
                    small: ImageInfo {
                        uri: row.get(3)?,
                        image: row.get(4)?,
                    },
                    normal: ImageInfo {
                        uri: row.get(5)?,
                        image: row.get(6)?,
                    },
                    large: ImageInfo {
                        uri: row.get(7)?,
                        image: row.get(8)?,
                    },
                    num_faces: row.get(9)?,
                })
            })?;
            Ok(card)
        })
        .await
        .map_err(|_| MessageError::SQLQuery)
    }

    pub async fn get_card(id: String) -> Result<Card, MessageError> {
        let card_info = Self::get_card_info(id.clone()).await?;
        if card_info.num_faces > 0 {
            let face = Self::get_card_face(card_info.id.clone(), 0).await?;
            return Ok(Card::art_series(
                card_info.id,
                card_info.name,
                face,
                0,
                card_info.num_faces,
            ));
        };
        let card = if let Some((blob, _)) = card_info.best_image() {
            Card::normal_card(card_info.id, card_info.name, card_info.cmc, blob)
        } else if let Some((uri, size)) = card_info.best_uri() {
            let image = Self::get_image(id, uri, size).await?;
            Card::normal_card(card_info.id, card_info.name, card_info.cmc, image)
        } else {
            Card::no_image_card(card_info.id, card_info.name, card_info.cmc)
        };
        Ok(card)
    }

    pub async fn next_card_face(id: String, current_face: usize) -> Result<Card, MessageError> {
        let card_info = Self::get_card_info(id.clone()).await?;
        let next_face = if current_face + 1 < card_info.num_faces {
            current_face + 1
        } else {
            0
        };
        let face = Self::get_card_face(id.clone(), next_face).await?;
        Ok(Card::art_series(
            id,
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
            .align_y(alignment::Vertical::Center)
            .height(height)
            .width(width),
        )
        .on_press(Message::CardClicked { card_id: self.id() })
        .into()
    }

    async fn get_image(id: String, url: String, size: ImageSize) -> Result<Vec<u8>, MessageError> {
        let image = download_image(url).await?;
        spawn(Database::write_card_image_blob(id, size, image.clone()));
        Ok(image)
    }

    async fn ensure_face_image(
        card_id: String,
        face_index: String,
        uri: String,
    ) -> Result<Option<Vec<u8>>, MessageError> {
        let conn = Database::connection()
            .await
            .map_err(|_| MessageError::SQLConnection)?;
        let image = download_image(uri).await?;
        let cloned_image = image.clone();
        conn.call(move |conn| {
            let mut stmt = conn.prepare(WRITE_FACE_SMALL_BLOB)?;
            stmt.execute(named_params! {
                ":card_id": card_id,
                ":face_index": face_index,
                ":small_blob": cloned_image,
            })?;
            Ok(())
        })
        .await
        .expect("Couldn' get face face image");
        // .map_err(|_| MessageError::SQLQuery)?;
        Ok(Some(image))
    }

    async fn get_card_face(
        card_id: String,
        face_index: usize,
    ) -> Result<Option<Vec<u8>>, MessageError> {
        let conn = Database::connection()
            .await
            .map_err(|_| MessageError::SQLConnection)?;

        let cloned_card_id = card_id.clone();
        let face_index = face_index.clone();
        let face_image_details = conn
            .clone()
            .call(move |conn| {
                let mut stmt = conn.prepare(GET_CARD_FACE)?;
                let card_face = stmt.query_row(
                    &[
                        (":card_id", &cloned_card_id),
                        (":face_index", &face_index.to_string()),
                    ],
                    |row| {
                        let small_uri: Option<String> = row.get(0)?;
                        let small_blob: Option<Vec<u8>> = row.get(1)?;
                        Ok((small_uri, small_blob, face_index))
                    },
                )?;
                Ok(card_face)
            })
            .await
            .expect("Query shouldn't fail");
        // .map_err(|_| MessageError::SQLQuery)?;
        match face_image_details {
            (_, Some(blob), _) => Ok(Some(blob)),
            (Some(uri), None, _) => {
                Self::ensure_face_image(card_id, face_index.to_string(), uri).await
            }
            (None, None, _) => Ok(None),
        }
    }
}

async fn download_image(url: String) -> Result<Vec<u8>, MessageError> {
    let response = reqwest::get(&url)
        .await
        .map_err(|_| MessageError::SQLQuery)?;
    let bytes = response.bytes().await.map_err(|_| MessageError::SQLQuery)?;
    Ok(bytes.to_vec())
}
