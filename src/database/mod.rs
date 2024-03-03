use std::path::PathBuf;

use anyhow::Context;
use bytes::Bytes;
use rusqlite::{
    named_params,
    types::{FromSql, ToSqlOutput, ValueRef},
    ToSql,
};

use crate::{
    card::card_data::ImageSize,
    db::{WRITE_FACE_SMALL_BLOB, WRITE_LARGE_IMAGE_BLOB, WRITE_SMALL_IMAGE_BLOB},
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
        let conn = Database::connection().await?;
        conn.call(move |conn| {
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
