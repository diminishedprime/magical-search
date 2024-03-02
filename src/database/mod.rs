use std::path::PathBuf;

use crate::{search::Search, CARDS_PER_ROW};

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
}
