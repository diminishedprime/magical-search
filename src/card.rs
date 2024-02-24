use crate::{db::{GET_CARD, WRITE_SMALL_IMAGE_BLOB}, MagicalSearch, Message, MessageError};
use iced::{
    futures::future::join,
    widget::{column, image::Handle, text, Image, button},
    Element,
};
use rusqlite::named_params;
use tokio_rusqlite::Connection;

// TODO - I should update the loaded card to be an enum that accounts for the
// different layouts. Dual faced cards have multiple images and will need a
// different data layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedCard {
    pub id: String,
    pub name: String,
    pub cmc: Option<f64>,
    pub image: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadingCard {
    pub id: String,
    pub name: String,
    pub cmc: Option<f64>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Card {
    LoadedImage(LoadedCard),
    LoadingImage(LoadingCard),
}

impl Card {
    pub fn fully_loaded(
        id: String,
        name: String,
        cmc: Option<f64>,
        image: Option<Vec<u8>>,
    ) -> Self {
        Self::LoadedImage(LoadedCard {
            id,
            name,
            cmc,
            image,
        })
    }
    pub fn loading(id: String, name: String, cmc: Option<f64>, image_url: Option<String>) -> Self {
        Self::LoadingImage(LoadingCard {
            id,
            name,
            cmc,
            image_url,
        })
    }
    pub fn id(&self) -> String {
        match self {
            Card::LoadedImage(loaded_card) => loaded_card.id.clone(),
            Card::LoadingImage(loading_card) => loading_card.id.clone(),
        }
    }
    pub fn name(&self) -> String {
        match self {
            Card::LoadedImage(loaded_card) => loaded_card.name.clone(),
            Card::LoadingImage(loading_card) => loading_card.name.clone(),
        }
    }
    pub async fn get_card(id: String, name: String) -> Result<Card, MessageError> {
        let path = MagicalSearch::db_path();
        let conn = Connection::open(path)
            .await
            .map_err(|_| MessageError::SQLConnection)?;

        let id = id.to_string();
        let mut card = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(GET_CARD)?;
                let card = stmt.query_row(&[(":id", &id)], |row| {
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
                })?;
                Ok(card)
            })
            .await
            .expect("Error getting card");
            // .map_err(|_| MessageError::SQLQuery)?;
        card.ensure_image().await?;
        Ok(card)
    }
    pub fn view(&self) -> Element<Message> {
        let button = button("Load Card").on_press(Message::CardClicked {
            card_id: self.id(),
        });
        match self {
            Card::LoadedImage(loaded_card) => match &loaded_card.image {
                Some(image) => column!(Image::new(Handle::from_memory(image.clone())), button).into(),
                None => column!(
                    text(loaded_card.name.clone()),
                    text(loaded_card.cmc.unwrap_or(0.0)),
                    button
                )
                .into(),
            },
            Card::LoadingImage(loading_card) => column!(
                text("Loading image"),
                text(loading_card.name.clone()),
                text(loading_card.cmc.unwrap_or(0.0))
            )
            .into(),
        }
    }

    async fn ensure_image(&mut self) -> Result<(), MessageError> {
        match self {
            Card::LoadingImage(loading) => {
                if let Some(image_url) = &loading.image_url {
                    let (conn, image) = join(
                        Connection::open(MagicalSearch::db_path()),
                        download_image(image_url.clone()),
                    )
                    .await;
                    let conn = conn.map_err(|_| MessageError::SQLConnection)?;
                    let image = image?;
                    let cloned_id = loading.id.clone();
                    let cloned_image = image.clone();
                    conn.call(move |conn| {
                        let mut stmt = conn.prepare(WRITE_SMALL_IMAGE_BLOB)?;
                        stmt.execute(named_params! {
                            ":card_id": cloned_id,
                            ":small_blob": cloned_image,
                        })?;
                        Ok(())
                    }).await
                    .unwrap();
                    // .map_err(|_| MessageError::SQLQuery)?;
                    *self = Card::fully_loaded(
                        loading.id.clone(),
                        loading.name.clone(),
                        loading.cmc,
                        Some(image),
                    );
                } else {
                    return Ok(());
                }
            }
            _ => (),
        };
        Ok(())
    }
}

async fn download_image(url: String) -> Result<Vec<u8>, MessageError> {
    let response = reqwest::get(&url)
        .await
        .map_err(|_| MessageError::SQLQuery)?;
    let bytes = response.bytes().await.map_err(|_| MessageError::SQLQuery)?;
    Ok(bytes.to_vec())
}
