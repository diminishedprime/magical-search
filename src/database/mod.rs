use std::{path::PathBuf, time::SystemTime};

use rusqlite::named_params;

use crate::{
    card::card_data::ImageSize,
    db::{WRITE_LARGE_IMAGE_BLOB, WRITE_SMALL_IMAGE_BLOB},
    search::Search,
    MessageError, CARDS_PER_ROW,
};

pub struct Database;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseErrors {
    #[error("Failed to connect to local database")]
    Connection(#[from] tokio_rusqlite::Error),
    #[error("TODO - fix me")]
    MessageError(#[from] crate::MessageError),
}

impl Database {
    fn path() -> PathBuf {
        PathBuf::from("target").join("cards.sqlite")
    }
    pub async fn connection() -> tokio_rusqlite::Result<tokio_rusqlite::Connection> {
        tokio_rusqlite::Connection::open(Database::path()).await
    }

    pub async fn fetch_card_ids(
        cursor: usize,
        search: Search,
    ) -> Result<Vec<String>, DatabaseErrors> {
        let conn = Database::connection().await?;
        let card_ids = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(&format!(
                    include_str!("get_ids_with_clauses.sql"),
                    clauses = search.to_clauses()
                ))?;
                let card_ids = stmt
                    .query_map(&[(":cursor", &cursor), (":limit", &CARDS_PER_ROW)], |row| {
                        let id: String = row.get(0)?;
                        Ok(id)
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(card_ids)
            })
            .await?;
        Ok(card_ids)
    }

    pub async fn write_card_image_blob(
        id: String,
        size: ImageSize,
        image: Vec<u8>,
    ) -> Result<(), DatabaseErrors> {
        println!("Current system time: {:?}", SystemTime::now());
        println!("Running the async work that's writing the blob in the background.");
        let conn = Database::connection().await?;
        let r = conn
            .call(move |conn| {
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
            .map_err(|_| MessageError::SQLQuery)?;
        println!("Current system time: {:?}", SystemTime::now());
        println!("Fully finished writing the blob in the background.");
        Ok(r)
    }
}
