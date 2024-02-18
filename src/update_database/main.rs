mod types;

use anyhow::{Context, Result};
use itertools::Itertools;
use reqwest::Client;
use rusqlite::params;
use serde::de::DeserializeOwned;

use serde_json::{self, Deserializer};
use std::fs::File;
use std::io::BufReader;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;
use types::Card;

static CARD_CHUNK_SIZE: usize = 50;
// Update this to true to update the card data from the Scryfall API

async fn run() -> Result<()> {
    let target_dir = PathBuf::from("target");
    let json_file_path = target_dir.join("cards.json");

    tokio::fs::create_dir_all(&target_dir)
        .await
        .with_context(|| format!("Failed to create directory: {:?}", &target_dir))?;

    if !json_file_path.exists() {
        download_json_from_api(&json_file_path).await?;
    }

    let file = File::open(&json_file_path)
        .with_context(|| format!("Failed to open JSON file: {:?}", &json_file_path))?;
    let reader = BufReader::new(file);

    let cards = iter_json_array::<Card, BufReader<_>>(reader);
    // Limit this to avoid unnecessarily hitting the API until I'm confident I have this working well.
    let card_chunks = cards.chunks(CARD_CHUNK_SIZE);

    let mut conn = rusqlite::Connection::open("cards.sqlite")?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            lang TEXT,
            object TEXT NOT NULL,
            layout TEXT,
            arena_id INTEGER,
            mtgo_id INTEGER,
            mtgo_foil_id INTEGER,
            tcgplayer_id INTEGER,
            tcgplayer_etched_id INTEGER,
            cardmarket_id INTEGER,
            oracle_id TEXT,
            prints_search_uri TEXT,
            rulings_uri TEXT,
            scryfall_uri TEXT,
            uri TEXT,
            cmc	Decimal(32,16),
            image_uri_small TEXT,
            image_uri_normal TEXT,
            image_uri_large TEXT,
            image_uri_png TEXT,
            image_uri_art_crop TEXT,
            image_uri_border_crop TEXT,
            image_data_small BLOB
        )",
        params![],
    )?;

    let mut tx = conn.transaction()?;
    for chunk in &card_chunks {
        let mut group = 0;
        for card in chunk {
            let card = card?;
            group = group + 1;
            {
                let mut stmt = tx.prepare(
                    "
        INSERT OR REPLACE INTO cards (
            id,
            name,
            lang,
            object,
            layout,
            arena_id,
            mtgo_id,
            mtgo_foil_id,
            tcgplayer_id,
            tcgplayer_etched_id,
            cardmarket_id,
            oracle_id,
            prints_search_uri,
            rulings_uri,
            scryfall_uri,
            uri,
            cmc,
            image_uri_small,
            image_uri_normal,
            image_uri_large,
            image_uri_png,
            image_uri_art_crop,
            image_uri_border_crop,
            image_data_small
        ) VALUES (
            ?, -- id
            ?, -- name
            ?, -- lang
            ?, -- object
            ?, -- layout
            ?, -- arena_id
            ?, -- mtgo_id
            ?, -- mtgo_foil_id
            ?, -- tcgplayer_id
            ?, -- tcgplayer_etched_id
            ?, -- cardmarket_id
            ?, -- oracle_id
            ?, -- prints_search_uri
            ?, -- rulings_uri
            ?, -- scryfall_uri
            ?, -- uri
            ?, -- cmc
            ?, -- image_uri_small
            ?, -- image_uri_normal
            ?, -- image_uri_large
            ?, -- image_uri_png
            ?, -- image_uri_art_crop
            ?, -- image_uri_border_crop
            ?  -- image_data_small
        )
    ",
                )?;

                let mut image_data = None;
                if let Some(image_uris) = &card.image_uris {
                    if let Some(small) = &image_uris.small {
                        image_data = download_image(&small).await.ok();
                    }
                }
                stmt.execute(params![
                    card.id,
                    card.name,
                    card.lang,
                    card.object,
                    card.layout,
                    card.arena_id,
                    card.mtgo_id,
                    card.mtgo_foil_id,
                    card.tcgplayer_id,
                    card.tcgplayer_etched_id,
                    card.cardmarket_id,
                    card.oracle_id,
                    card.prints_search_uri,
                    card.rulings_uri,
                    card.scryfall_uri,
                    card.uri,
                    card.cmc,
                    card.image_uris.as_ref().map(|u| u.small.clone()),
                    card.image_uris.as_ref().map(|u| u.normal.clone()),
                    card.image_uris.as_ref().map(|u| u.large.clone()),
                    card.image_uris.as_ref().map(|u| u.png.clone()),
                    card.image_uris.as_ref().map(|u| u.art_crop.clone()),
                    card.image_uris.as_ref().map(|u| u.border_crop.clone()),
                    image_data
                ])?;
            }
        }
        tx.commit()?;
        println!("Successfully imported {} cards into the database.", group);
        println!("Waiting for the next batch...");
        tx = conn.transaction()?;
        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn download_json_from_api(path: &Path) -> Result<()> {
    // Fetch the JSON data from the API
    let client = Client::new();
    // TODO - Ideally this is
    let response = client
        .get("https://data.scryfall.io/all-cards/all-cards-20240217101517.json")
        .send()
        .await
        .context("Failed to send HTTP request")?;

    // Create a file and write the JSON data to it
    let mut file = tokio::fs::File::create(&path)
        .await
        .with_context(|| format!("Failed to create file: {:?}", &path))?;
    tokio::io::copy(&mut response.bytes().await?.as_ref(), &mut file)
        .await
        .context("Failed to write JSON data to file")?;

    Ok(())
}

async fn download_image(url: &str) -> Result<Vec<u8>, String> {
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Error fetching image: {:?}", &e))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Error fetching image: {:?}", &e))?
        .to_vec();
    Ok(bytes)
}

fn read_skipping_ws(mut reader: impl Read) -> io::Result<u8> {
    loop {
        let mut byte = 0u8;
        reader.read_exact(std::slice::from_mut(&mut byte))?;
        if !byte.is_ascii_whitespace() {
            return Ok(byte);
        }
    }
}

fn invalid_data(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

fn deserialize_single<T: DeserializeOwned, R: Read>(reader: R) -> io::Result<T> {
    let next_obj = Deserializer::from_reader(reader).into_iter::<T>().next();
    match next_obj {
        Some(result) => result.map_err(Into::into),
        None => Err(invalid_data("premature EOF")),
    }
}

fn yield_next_obj<T: DeserializeOwned, R: Read>(
    mut reader: R,
    at_start: &mut bool,
) -> io::Result<Option<T>> {
    if !*at_start {
        *at_start = true;
        if read_skipping_ws(&mut reader)? == b'[' {
            // read the next char to see if the array is empty
            let peek = read_skipping_ws(&mut reader)?;
            if peek == b']' {
                Ok(None)
            } else {
                deserialize_single(io::Cursor::new([peek]).chain(reader)).map(Some)
            }
        } else {
            Err(invalid_data("`[` not found"))
        }
    } else {
        match read_skipping_ws(&mut reader)? {
            b',' => deserialize_single(reader).map(Some),
            b']' => Ok(None),
            _ => Err(invalid_data("`,` or `]` not found")),
        }
    }
}

pub fn iter_json_array<T: DeserializeOwned, R: Read>(
    mut reader: R,
) -> impl Iterator<Item = Result<T, io::Error>> {
    let mut at_start = false;
    std::iter::from_fn(move || yield_next_obj(&mut reader, &mut at_start).transpose())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
