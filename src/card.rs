use {
    crate::{
        db::{GET_CARD, GET_CARD_FACE, WRITE_FACE_SMALL_BLOB, WRITE_SMALL_IMAGE_BLOB},
        MagicalSearch, Message, MessageError,
    },
    iced::{
        futures::future::join,
        widget::{button, column, image::Handle, row, text, Image},
        Command, Element,
    },
    rusqlite::named_params,
    tokio_rusqlite::Connection,
};

#[derive(Debug, Clone)]
pub struct NormalCard {
    pub id: String,
    pub name: String,
    pub cmc: Option<f64>,
    pub image: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ArtSeries {
    pub id: String,
    pub name: String,
    pub selected_face: usize,
    pub face: Option<Vec<u8>>,
    pub num_faces: usize,
}

#[derive(Debug, Clone)]
pub enum LoadedCard {
    Normal(NormalCard),
    ArtSeries(ArtSeries),
}

#[derive(Debug, Clone)]
pub struct LoadingCard {
    pub id: String,
    pub name: String,
    pub cmc: Option<f64>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Card {
    Loaded(LoadedCard),
    Loading { id: String, url: String },
}

impl Card {
    pub fn is_loading(&self) -> bool {
        matches!(self, Card::Loading { .. })
    }

    pub fn load_action(&self) -> Command<Message> {
        if let Card::Loading { id, .. } = self {
            Command::perform(Card::get_card(id.to_string()), Message::CardLoaded)
        } else {
            Command::none()
        }
    }

    pub fn normal_card(id: String, name: String, cmc: Option<f64>, image: Option<Vec<u8>>) -> Self {
        Self::Loaded(LoadedCard::Normal(NormalCard {
            id,
            name,
            cmc,
            image,
        }))
    }

    pub fn art_series(
        id: String,
        name: String,
        face: Option<Vec<u8>>,
        selected_face: usize,
        num_faces: usize,
    ) -> Self {
        Self::Loaded(LoadedCard::ArtSeries(ArtSeries {
            id,
            name,
            face,
            selected_face,
            num_faces,
        }))
    }

    pub fn loading(id: String, url: String) -> Self {
        Self::Loading { id, url }
    }

    pub fn id(&self) -> String {
        match self {
            Card::Loaded(loaded_card) => match loaded_card {
                LoadedCard::Normal(normal) => normal.id.clone(),
                LoadedCard::ArtSeries(art_series) => art_series.id.clone(),
            },
            Card::Loading { id, .. } => id.clone(),
        }
    }

    pub async fn get_card_info(
        id: String,
    ) -> Result<
        (
            String,
            String,
            Option<f64>,
            Option<String>,
            Option<Vec<u8>>,
            usize,
        ),
        MessageError,
    > {
        let path = MagicalSearch::db_path();
        let conn = Connection::open(path)
            .await
            .map_err(|_| MessageError::SQLConnection)?;

        let id = id.to_string();
        conn.call(move |conn| {
            let mut stmt = conn.prepare(GET_CARD)?;
            let card = stmt.query_row(&[(":id", &id)], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let cmc: Option<f64> = row.get(2)?;
                let url: Option<String> = row.get(3)?;
                let image: Option<Vec<u8>> = row.get(4)?;
                let num_faces: usize = row.get(5)?;
                Ok((id, name, cmc, url, image, num_faces))
            })?;
            Ok(card)
        })
        .await
        .map_err(|_| MessageError::SQLQuery)
    }

    pub async fn get_card(id: String) -> Result<Card, MessageError> {
        let (id, name, cmc, url, image, num_faces) = Self::get_card_info(id).await?;
        let mut card = if num_faces > 0 {
            let face = Self::get_card_face(id.clone(), 0).await?;
            return Ok(Card::art_series(id, name, face, 0, num_faces));
        } else if url.is_some() && image.is_none() {
            Card::loading(id, url.unwrap())
        } else {
            Card::normal_card(id, name, cmc, image)
        };
        card.ensure_image().await?;
        Ok(card)
    }

    pub async fn next_card_face(id: String, current_face: usize) -> Result<Card, MessageError> {
        let (id, name, _, _, _, num_faces) = Self::get_card_info(id.clone()).await?;
        let next_face = if current_face + 1 < num_faces {
            current_face + 1
        } else {
            0
        };
        let face = Self::get_card_face(id.clone(), next_face).await?;
        Ok(Card::art_series(id, name, face, next_face, num_faces))
    }

    pub fn view(&self) -> Element<Message> {
        match self {
            Card::Loaded(loaded_card) => {
                let view_detail = button("Load Card").on_press(Message::CardClicked {
                    card_id: self.id().clone(),
                });
                match loaded_card {
                    LoadedCard::Normal(loaded_card) => match &loaded_card.image {
                        Some(image) => column!(
                            Image::new(Handle::from_memory(image.clone()))
                                .content_fit(iced::ContentFit::None),
                            view_detail
                        )
                        .into(),
                        None => column!(
                            text(loaded_card.name.clone()),
                            text(loaded_card.cmc.unwrap_or(0.0)),
                            view_detail
                        )
                        .into(),
                    },
                    LoadedCard::ArtSeries(ArtSeries { face, .. }) => {
                        let controls = row!(
                            view_detail,
                            button("Flip Card").on_press(Message::NextFace {
                                card_id: self.id().clone()
                            })
                        );
                        match face {
                            Some(image) => {
                                column!(Image::new(Handle::from_memory(image.clone())), controls)
                            }
                            None => column!(text("No image for this face."), controls),
                        }
                        .into()
                    }
                }
            }
            Card::Loading { .. } => column!(text("Loading card"),).into(),
        }
    }

    async fn write_small_blob(
        conn: &Connection,
        id: String,
        image: Vec<u8>,
    ) -> Result<(), MessageError> {
        conn.call(move |conn| {
            let mut stmt = conn.prepare(WRITE_SMALL_IMAGE_BLOB)?;
            stmt.execute(named_params! {
                ":card_id": id,
                ":small_blob": image,
            })?;
            Ok(())
        })
        .await
        .map_err(|_| MessageError::SQLQuery)
    }

    async fn ensure_image(&mut self) -> Result<(), MessageError> {
        if let Card::Loading { id, url } = self {
            let (conn, image) = join(
                Connection::open(MagicalSearch::db_path()),
                download_image(url.clone()),
            )
            .await;
            let conn = conn.map_err(|_| MessageError::SQLConnection)?;
            let image = image?;
            Self::write_small_blob(&conn, id.clone(), image.clone()).await?;
            let (id, name, cmc, _, _, _) = Self::get_card_info(id.clone()).await?;
            *self = Card::normal_card(id.clone(), name.clone(), cmc, Some(image));
        }
        Ok(())
    }

    async fn ensure_face_image(
        card_id: String,
        face_index: String,
        uri: String,
    ) -> Result<Option<Vec<u8>>, MessageError> {
        let conn = Connection::open(MagicalSearch::db_path())
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
        let conn = Connection::open(MagicalSearch::db_path())
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
