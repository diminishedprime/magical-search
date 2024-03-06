use std::path::PathBuf;

use anyhow::Context;
use bytes::Bytes;
use rusqlite::{
    named_params,
    types::{FromSql, ToSqlOutput, ValueRef},
    ToSql,
};

use crate::{
    card::card_data::{CardData, ImageInfo, ImageSize},
    db::{GET_CARD, WRITE_FACE_SMALL_BLOB, WRITE_LARGE_IMAGE_BLOB, WRITE_SMALL_IMAGE_BLOB},
    search::Search,
    CARDS_PER_ROW,
};

pub struct Database;

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
    ) -> Result<Vec<String>, anyhow::Error> {
        println!("Search: {:?}", search);
        let conn = Database::connection().await?;
        conn.call(move |conn| {
            let sql = format!(
                include_str!("get_ids_with_clauses.sql"),
                clauses = search
                    .parsed_search
                    .map(|s| s.to_clauses())
                    .unwrap_or(String::new()) // clauses = search.to_clauses()
            );
            let mut stmt = conn.prepare(&sql)?;
            let card_ids = stmt
                .query_map(&[(":cursor", &cursor), (":limit", &CARDS_PER_ROW)], |row| {
                    let id: String = row.get(0)?;
                    Ok(id)
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(card_ids)
        })
        .await
        .context("failed to fetch card ids.")
    }

    pub async fn write_card_image_blob(
        id: String,
        size: ImageSize,
        image: Bytes,
    ) -> Result<(), anyhow::Error> {
        let conn = Database::connection().await?;
        conn.call(move |conn| {
            match size {
                ImageSize::Small => {
                    let mut stmt = conn.prepare(WRITE_SMALL_IMAGE_BLOB)?;
                    stmt.execute(named_params! {
                        ":card_id": id,
                        ":small_blob": BytesWrapper(image),
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
                        ":large_blob": BytesWrapper(image),
                    })?;
                }
            }
            Ok(())
        })
        .await
        .context("Failed to write image blob")
    }

    pub async fn write_face_image_blob(
        card_id: String,
        face_index: String,
        size: ImageSize,
        image: Bytes,
    ) -> Result<(), anyhow::Error> {
        let conn = Database::connection().await?;
        conn.call(move |conn| {
            let mut stmt = conn.prepare(WRITE_FACE_SMALL_BLOB)?;
            match size {
                ImageSize::Small => stmt.execute(named_params! {
                    ":card_id": card_id,
                    ":face_index": face_index,
                    ":small_blob": BytesWrapper(image),
                })?,
                ImageSize::Medium => todo!(),
                ImageSize::Large => todo!(),
            };
            Ok(())
        })
        .await
        .context("Couldn't write face image")
    }

    pub async fn get_card_info(id: String) -> Result<CardData, anyhow::Error> {
        let conn = Database::connection().await?;

        let cloned_id = id.to_string();
        conn.call(move |conn| {
            let mut stmt = conn.prepare(GET_CARD)?;
            let card = stmt.query_row(&[(":id", &cloned_id)], |row| {
                Ok(CardData {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cmc: row.get(2)?,
                    small: ImageInfo {
                        uri: row.get(3)?,
                        image: row
                            .get::<_, Option<BytesWrapper>>(4)
                            .map(|b| b.map(|b| b.0))?,
                    },
                    normal: ImageInfo {
                        uri: row.get(5)?,
                        image: row
                            .get::<_, Option<BytesWrapper>>(6)
                            .map(|b| b.map(|b| b.0))?,
                    },
                    large: ImageInfo {
                        uri: row.get(7)?,
                        image: row
                            .get::<_, Option<BytesWrapper>>(8)
                            .map(|b| b.map(|b| b.0))?,
                    },
                    num_faces: row.get(9)?,
                })
            })?;
            Ok(card)
        })
        .await
        .context(format!("Couldn't get card info for id: {}", id))
    }
}

pub struct BytesWrapper(pub Bytes);

impl FromSql for BytesWrapper {
    fn column_result(value: ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            // TODO - Null makes me think maybe I should be implementing an
            // Option<BytesWrapper> or something instead?
            ValueRef::Null | ValueRef::Integer(_) | ValueRef::Real(_) | ValueRef::Text(_) => {
                Err(rusqlite::types::FromSqlError::InvalidType)
            }
            ValueRef::Blob(b) => Ok(BytesWrapper(Bytes::copy_from_slice(b))),
        }
    }
}

impl ToSql for BytesWrapper {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Blob(&self.0)))
    }
}
