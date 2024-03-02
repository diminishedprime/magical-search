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

    fn fetch_card_ids_sql(search: &Search) -> String {
        format!(
            r#"
SELECT cards.id
FROM cards
{clauses}
LIMIT :limit
OFFSET :cursor;
"#,
            clauses = search.to_clauses()
        )
    }

    pub async fn fetch_card_ids(
        cursor: usize,
        search: Search,
    ) -> Result<Vec<String>, DatabaseErrors> {
        let conn = Database::connection().await?;
        let card_ids = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(&Self::fetch_card_ids_sql(&search))?;
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
