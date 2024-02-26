use std::path::PathBuf;

use iced::futures::{stream::FuturesOrdered, TryStreamExt};

use crate::{card::Card, cards::Cards, search::Search, to_sql::ToSql, LIMIT};

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

    fn fetch_cards_with_search_sql(search: Search) -> String {
        let mut clauses = search.to_sql();
        if !clauses.trim().is_empty() {
            clauses = format!(
                r#"
WHERE
{clauses}
"#,
                clauses = clauses
            )
            .trim()
            .to_string();
        };
        format!(
            r#"
SELECT 
    cards.id,
    cards.name,
    cards.cmc,
    card_image_uris.small AS small_image_url,
    card_image_blobs.small AS small_image_blob
FROM 
    cards
LEFT JOIN 
    card_image_uris ON cards.id = card_image_uris.card_id
LEFT JOIN 
    card_image_blobs ON cards.id = card_image_blobs.card_id
{clauses}
LIMIT :limit;
        "#,
            clauses = clauses,
        )
    }

    pub async fn fetch_cards_with_search(search: Search) -> Result<Cards, DatabaseErrors> {
        let overall_start_time = std::time::Instant::now();
        let query = Self::fetch_cards_with_search_sql(search);
        println!("{}", query);

        let conn = Database::connection().await?;

        let cards = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(&query)?;
                let cards = stmt
                    .query_map(&[(":limit", &LIMIT.to_string())], |row| {
                        let id: String = row.get(0)?;
                        Ok(id)
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(cards)
            })
            .await?;
        let query_id_time = overall_start_time.elapsed();
        println!("Query ID took {} milliseconds", query_id_time.as_millis());

        let cards: Vec<Card> = cards
            .into_iter()
            .map(|id| Card::get_card(id))
            .collect::<FuturesOrdered<_>>()
            .try_collect()
            .await?;
        let card_subquery_time = overall_start_time.elapsed();
        println!(
            "Card subquery took {} milliseconds",
            card_subquery_time.as_millis()
        );

        let finish_time = overall_start_time.elapsed();
        println!("Query took {} milliseconds", finish_time.as_millis());
        Ok(Cards::new(cards))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search;

    #[tokio::test]
    pub async fn fetch_cards_with_equals() {
        let search = search::search("c=W").unwrap();
        let query = Database::fetch_cards_with_search_sql(search);
        let conn = Database::connection().await.unwrap();
        let card_ids = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(&query).unwrap();
                let card_ids = stmt
                    .query_map(&[(":limit", &LIMIT.to_string())], |row| {
                        let id: String = row.get(0)?;
                        Ok(id)
                    })
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                Ok(card_ids)
            })
            .await
            .unwrap();
        for id in card_ids {
            let (colors, name): ((bool, bool, bool, bool, bool, bool), String) = conn
                .call(move |conn| {
                    let mut stmt = conn
                        .prepare("SELECT C,W,U,B,R,G, name FROM cards WHERE cards.id = :card_id;")
                        .unwrap();
                    let colors = stmt
                        .query_row(&[(":card_id", &id)], |row| {
                            Ok((
                                (
                                    row.get(0)?,
                                    row.get(1)?,
                                    row.get(2)?,
                                    row.get(3)?,
                                    row.get(4)?,
                                    row.get(5)?,
                                ),
                                row.get(6)?,
                            ))
                        })
                        .unwrap();
                    Ok(colors)
                })
                .await
                .unwrap();
            assert_eq!(
                colors,
                (false, true, false, false, false, false),
                "{}",
                name
            );
        }
    }
}
