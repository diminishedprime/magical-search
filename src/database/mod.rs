use std::path::PathBuf;
pub mod to_sql;

use anyhow::Context;
use bytes::Bytes;
use rusqlite::{
    named_params,
    types::{FromSql, ToSqlOutput, ValueRef},
    ToSql,
};
use tokio::spawn;

use self::to_sql::{ToSql as _, SQL};
use crate::{card::card_data::CardData, search::Search, CARDS_PER_ROW};

pub struct Database;

impl Database {
    fn path() -> PathBuf {
        PathBuf::from("target").join("cards.sqlite")
    }
    pub async fn connection() -> tokio_rusqlite::Result<tokio_rusqlite::Connection> {
        tokio_rusqlite::Connection::open(Database::path()).await
    }

    pub async fn get_card_face(
        card_id: String,
        face_index: usize,
    ) -> Result<Option<Bytes>, anyhow::Error> {
        let conn = Self::connection().await?;

        let cloned_card_id = card_id.clone();
        let face_index = face_index.clone();
        let (small_uri, normal_uri, large_uri, blob) = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(include_str!("get_card_face.sql"))?;
                let card_face = stmt.query_row(
                    &[
                        (":card_id", &cloned_card_id),
                        (":face_index", &face_index.to_string()),
                    ],
                    |row| {
                        let small_uri: Option<String> = row.get(0)?;
                        let normal_uri: Option<String> = row.get(1)?;
                        let large_uri: Option<String> = row.get(2)?;
                        let blob: Option<BytesWrapper> = row.get(3)?;
                        Ok((small_uri, normal_uri, large_uri, blob))
                    },
                )?;
                Ok(card_face)
            })
            .await
            .context("Query shouldn't fail")?;
        if let Some(blob) = blob {
            Ok(Some(blob.0))
        } else {
            let uri = large_uri.or(normal_uri).or(small_uri);
            if let Some(uri) = uri {
                let blob = download_image(uri).await?;
                spawn(Database::write_card_face_image_blob(
                    card_id.clone(),
                    face_index,
                    blob.clone(),
                ));
                Ok(Some(blob))
            } else {
                Ok(None)
            }
        }
    }

    fn fetch_card_ids_sql(search: Search) -> String {
        let s = search
            .parsed_search
            .map(|s| s.to_sql())
            .unwrap_or(SQL::default());
        let sql = format!(
            include_str!("get_ids_with_clauses.sql"),
            joins = s.joins(),
            clauses = s.wheres()
        );
        // println!("{}", sql);
        sql
    }

    pub async fn fetch_card_ids(
        cursor: usize,
        search: Search,
    ) -> Result<Vec<String>, anyhow::Error> {
        let conn = Database::connection().await?;
        conn.call(move |conn| {
            let mut stmt = conn.prepare(&Self::fetch_card_ids_sql(search))?;
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

    async fn write_card_face_image_blob(
        card_id: String,
        face_index: usize,
        blob: Bytes,
    ) -> Result<(), anyhow::Error> {
        let conn = Self::connection().await?;
        conn.call(move |conn| {
            let mut stmt = conn.prepare(include_str!("write_card_face_image.sql"))?;
            stmt.execute(named_params! {
                ":card_id": card_id,
                ":face_index": face_index,
                ":image": BytesWrapper(blob),
            })?;
            Ok(())
        })
        .await
        .context("Couldn't write face image")?;
        Ok(())
    }

    pub async fn write_card_image(card_id: String, image: Bytes) -> Result<(), anyhow::Error> {
        let conn = Database::connection().await?;
        conn.call(move |conn| {
            let mut stmt = conn.prepare(include_str!("write_card_image.sql"))?;
            stmt.execute(named_params! {
                ":card_id": card_id,
                ":image": BytesWrapper(image),
            })?;
            Ok(())
        })
        .await
        .context("Failed to write image blob")
    }

    pub async fn get_card_data(card_id: String) -> Result<CardData, anyhow::Error> {
        let conn = Database::connection().await?;

        let cloned_id = card_id.to_string();
        let mut card_data = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(include_str!("get_card_info.sql"))?;
                let card_data = stmt.query_row(&[(":id", &cloned_id)], |row| {
                    let image: Option<BytesWrapper> = row.get(6)?;
                    Ok(CardData {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        cmc: row.get(2)?,
                        small_uri: row.get(3)?,
                        normal_uri: row.get(4)?,
                        large_uri: row.get(5)?,
                        image: image.map(|b| b.0),
                        num_faces: row.get(7)?,
                        oracle_text: row.get(8)?,
                    })
                })?;
                Ok(card_data)
            })
            .await
            .context(format!("Couldn't get card info for id: {}", card_id))?;

        // TODO - Decide if I want to do this as a separate action, or block this action completing.
        if let Some(uri) = card_data.best_uri() {
            if card_data.image.is_none() {
                let blob = download_image(uri).await?;
                // TODO - I should eventually have better error handling on
                // this. Right now this fails silently if something goes wrong.
                // I don't want this to block so I probably need some channels
                // or something to indicate if something is being goofed.
                spawn(Database::write_card_image(card_id.clone(), blob.clone()));
                card_data.image = Some(blob);
            }
        }
        Ok(card_data)
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

async fn download_image(url: String) -> Result<Bytes, anyhow::Error> {
    let response = reqwest::get(&url).await.context("Failed to get image")?;
    response
        .bytes()
        .await
        .context("Failed to bytes()-ify request")
}

impl ToSql for BytesWrapper {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Blob(&self.0)))
    }
}

#[cfg(test)]
mod test {
    use crate::search::Search;

    #[test]
    fn empty_typeline_does_not_show_up() {
        let actual = super::Database::fetch_card_ids_sql(Search::from("t:"));
        assert!(
            !actual.contains("cards.type_line LIKE"),
            "An empty typeline shouldn't influence the query."
        );
    }
}
