use iced::{
    alignment::{self},
    futures::future::join,
    widget::{button, column, container, image::Handle, text, Image},
    Command, Element,
};
use rusqlite::named_params;
use tokio_rusqlite::Connection;

use crate::{
    database::Database,
    db::{
        GET_CARD, GET_CARD_FACE, WRITE_FACE_SMALL_BLOB, WRITE_LARGE_IMAGE_BLOB,
        WRITE_SMALL_IMAGE_BLOB,
    },
    Message, MessageError,
};

#[derive(Debug, Clone)]
pub enum ImageSize {
    Small,
    Medium,
    Large,
}

struct ImageInfo {
    uri: Option<String>,
    image: Option<Vec<u8>>,
}

pub struct CardInfo {
    id: String,
    name: String,
    cmc: Option<f64>,
    small: ImageInfo,
    normal: ImageInfo,
    large: ImageInfo,
    num_faces: usize,
}

impl CardInfo {
    pub fn best_uri(&self) -> Option<(String, ImageSize)> {
        self.large
            .uri
            .as_ref()
            .map(|uri| (uri.clone(), ImageSize::Large))
            .or(self
                .normal
                .uri
                .as_ref()
                .map(|uri| (uri.clone(), ImageSize::Medium)))
            .or(self
                .small
                .uri
                .as_ref()
                .map(|uri| (uri.clone(), ImageSize::Small)))
    }
    pub fn best_image(&self) -> Option<(Vec<u8>, ImageSize)> {
        self.large
            .image
            .clone()
            .map(|image| (image, ImageSize::Large))
            .or(self
                .normal
                .image
                .clone()
                .map(|image| (image, ImageSize::Medium)))
            .or(self
                .small
                .image
                .clone()
                .map(|image| (image, ImageSize::Small)))
    }
}

#[derive(Debug, Clone)]
pub struct NormalCard {
    pub id: String,
    pub name: String,
    pub cmc: Option<f64>,
    pub image: Option<Vec<u8>>,
}

impl NormalCard {
    fn view(&self) -> Element<Message> {
        match &self.image {
            Some(image) => column!(Image::new(Handle::from_memory(image.clone()))
                .content_fit(iced::ContentFit::Contain),),
            None => column!(text(self.name.clone()), text(self.cmc.unwrap_or(0.0)),),
        }
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct ArtSeries {
    pub id: String,
    pub name: String,
    pub selected_face: usize,
    pub face: Option<Vec<u8>>,
    pub num_faces: usize,
}

impl ArtSeries {
    fn view(&self) -> Element<Message> {
        match &self.face {
            Some(image) => {
                column!(Image::new(Handle::from_memory(image.clone())))
            }
            None => column!(text("No image for this face.")),
        }
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct LoadingCard {
    pub id: String,
    pub url: String,
    pub size: ImageSize,
}

impl LoadingCard {
    fn view(&self) -> Element<Message> {
        column!(text("Loading card"),).into()
    }
}

#[derive(Debug, Clone)]
pub enum Card {
    Normal(NormalCard),
    ArtSeries(ArtSeries),
    Loading(LoadingCard),
}

impl Card {
    pub fn is_loading(&self) -> bool {
        matches!(self, Card::Loading { .. })
    }

    pub fn load_action(&self) -> Command<Message> {
        if let Card::Loading(LoadingCard { id, .. }) = self {
            Command::perform(Card::get_card(id.to_string()), Message::CardLoaded)
        } else {
            Command::none()
        }
    }

    pub fn normal_card(id: String, name: String, cmc: Option<f64>, image: Option<Vec<u8>>) -> Self {
        Self::Normal(NormalCard {
            id,
            name,
            cmc,
            image,
        })
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

    pub fn loading(id: String, url: String, size: ImageSize) -> Self {
        Self::Loading(LoadingCard { id, url, size })
    }

    pub fn id(&self) -> String {
        match self {
            Card::Normal(normal) => normal.id.clone(),
            Card::ArtSeries(art_series) => art_series.id.clone(),
            Card::Loading(LoadingCard { id, .. }) => id.clone(),
        }
    }

    pub async fn get_card_info(id: String) -> Result<CardInfo, MessageError> {
        let conn = Database::connection()
            .await
            .map_err(|_| MessageError::SQLConnection)?;

        let id = id.to_string();
        conn.call(move |conn| {
            let mut stmt = conn.prepare(GET_CARD)?;
            let card = stmt.query_row(&[(":id", &id)], |row| {
                Ok(CardInfo {
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
        let card_info = Self::get_card_info(id).await?;
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
        let mut card = if let Some((blob, _)) = card_info.best_image() {
            Card::normal_card(card_info.id, card_info.name, card_info.cmc, Some(blob))
        } else if let Some((uri, size)) = card_info.best_uri() {
            Card::loading(card_info.id, uri, size)
        } else {
            Card::normal_card(card_info.id, card_info.name, card_info.cmc, None)
        };
        card.ensure_image().await?;
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
        let height = 210;
        let width = 150;
        button(
            container(match self {
                Card::Normal(normal) => normal.view(),
                Card::ArtSeries(art_series) => art_series.view(),
                Card::Loading(loading_card) => loading_card.view(),
            })
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .height(height)
            .width(width),
        )
        .on_press(Message::CardClicked { card_id: self.id() })
        .into()
    }

    async fn write_blob(
        conn: &Connection,
        id: String,
        size: ImageSize,
        image: Vec<u8>,
    ) -> Result<(), MessageError> {
        conn.call(move |conn| {
            match size {
                ImageSize::Small => {
                    let mut stmt = conn.prepare(WRITE_SMALL_IMAGE_BLOB)?;
                    stmt.execute(named_params! {
                        ":card_id": id,
                        ":small_blob": image,
                    })?;
                }
                ImageSize::Medium => {
                    // TODO - write medium blob
                    unimplemented!("Medium blob not implemented")
                }
                ImageSize::Large => {
                    let mut stmt = conn.prepare(WRITE_LARGE_IMAGE_BLOB)?;
                    stmt.execute(named_params! {
                        ":card_id": id,
                        ":large_blob": image,
                    })?;
                }
            }
            Ok(())
        })
        .await
        .map_err(|_| MessageError::SQLQuery)
    }

    async fn ensure_image(&mut self) -> Result<(), MessageError> {
        if let Card::Loading(LoadingCard { id, url, size }) = self {
            let (conn, image) = join(Database::connection(), download_image(url.clone())).await;
            let conn = conn.map_err(|_| MessageError::SQLConnection)?;
            let image = image?;
            Self::write_blob(&conn, id.clone(), size.clone(), image.clone()).await?;
            let card_info = Self::get_card_info(id.clone()).await?;
            *self = Card::normal_card(
                id.clone(),
                card_info.name.clone(),
                card_info.cmc,
                Some(image),
            );
        }
        Ok(())
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
