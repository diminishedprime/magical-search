mod types;

use anyhow::{Context, Result};
use itertools::Itertools;
use reqwest::Client;
use rusqlite::named_params;
use serde::de::DeserializeOwned;
use serde_json::{self, Deserializer};
use std::fs::File;
use std::io::BufReader;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use types::Card;

static CARD_CHUNK_SIZE: usize = 1000;
// Update this to true to update the card data from the Scryfall API

static CREATE_TABLE_SQL: &str = r#"
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
    cmc DECIMAL(32,16),
    power TEXT,
    toughness TEXT,
    flavor_text TEXT,
    oracle_text TEXT
);

CREATE TABLE IF NOT EXISTS card_colors (
    card_id TEXT,
    color TEXT,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);

CREATE TABLE IF NOT EXISTS card_image_uris (
    card_id TEXT,
    small TEXT,
    normal TEXT,
    large TEXT,
    png TEXT,
    art_crop TEXT,
    border_crop TEXT,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);

CREATE TABLE IF NOT EXISTS card_color_identity (
    card_id TEXT,
    color_identity TEXT,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);

CREATE TABLE IF NOT EXISTS card_keywords (
    card_id TEXT,
    keyword TEXT,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);
"#;

static ADD_CARD_SQL: &str = r#"
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
    power,
    toughness,
    flavor_text,
    oracle_text
) VALUES (
    :id,
    :name,
    :lang,
    :object,
    :layout,
    :arena_id,
    :mtgo_id,
    :mtgo_foil_id,
    :tcgplayer_id,
    :tcgplayer_etched_id,
    :cardmarket_id,
    :oracle_id,
    :prints_search_uri,
    :rulings_uri,
    :scryfall_uri,
    :uri,
    :cmc,
    :power,
    :toughness,
    :flavor_text,
    :oracle_text
)
"#;

const ADD_CARD_COLORS_SQL: &str = r#"
INSERT OR REPLACE INTO card_colors (
    card_id,
    color
) VALUES (
    :card_id,
    :color
)
"#;

const ADD_KEYWORDS_SQL: &str = r#"
INSERT OR REPLACE INTO card_keywords (
    card_id,
    keyword
) VALUES (
    :card_id,
    :keyword
)
"#;

const ADD_COLOR_IDENTITY_SQL: &str = r#"
INSERT OR REPLACE INTO card_color_identity (
    card_id,
    color_identity
) VALUES (
    :card_id,
    :color_identity
)
"#;

const ADD_IMAGE_URIS_SQL: &str = r#"
INSERT OR REPLACE INTO card_image_uris (
    card_id,
    small,
    normal,
    large,
    png,
    art_crop,
    border_crop
) VALUES (
    :card_id,
    :small,
    :normal,
    :large,
    :png,
    :art_crop,
    :border_crop
)
"#;

async fn run() -> Result<()> {
    let target_dir = PathBuf::from("target");
    let json_file_path = target_dir.join("cards.json");
    let cards_db_file_path = target_dir.join("cards.sqlite");

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
    let card_chunks = cards.chunks(CARD_CHUNK_SIZE);

    let mut conn = rusqlite::Connection::open(cards_db_file_path)?;

    conn.execute_batch(CREATE_TABLE_SQL)?;

    let mut total_cards = 0;
    for chunk in &card_chunks {
        let mut group = 0;
        let mut tx = conn.transaction()?;
        for card in chunk {
            let card = card?;
            group = group + 1;
            {
                tx.execute(
                    ADD_CARD_SQL,
                    named_params! {
                    ":id": card.id,
                    ":name": card.name,
                    ":lang": card.lang,
                    ":object": card.object,
                    ":layout": card.layout,
                    ":arena_id": card.arena_id,
                    ":mtgo_id": card.mtgo_id,
                    ":mtgo_foil_id": card.mtgo_foil_id,
                    ":tcgplayer_id": card.tcgplayer_id,
                    ":tcgplayer_etched_id": card.tcgplayer_etched_id,
                    ":cardmarket_id": card.cardmarket_id,
                    ":oracle_id": card.oracle_id,
                    ":prints_search_uri": card.prints_search_uri,
                    ":rulings_uri": card.rulings_uri,
                    ":scryfall_uri": card.scryfall_uri,
                    ":uri": card.uri,
                    ":cmc": card.cmc,
                    ":power": card.power,
                    ":toughness": card.toughness,
                    ":flavor_text": card.flavor_text,
                    ":oracle_text": card.oracle_text,
                    },
                )?;
                if let Some(colors) = card.colors {
                    for color in colors {
                        tx.execute(
                            ADD_CARD_COLORS_SQL,
                            named_params! {
                                ":card_id": &card.id,
                                ":color": color,
                            },
                        )?;
                    }
                }
                for keyword in &card.keywords {
                    tx.execute(
                        ADD_KEYWORDS_SQL,
                        named_params! {
                            ":card_id": &card.id,
                            ":keyword": keyword,
                        },
                    )?;
                }
                for color_identity in &card.color_identity {
                    tx.execute(
                        ADD_COLOR_IDENTITY_SQL,
                        named_params! {
                            ":card_id": &card.id,
                            ":color_identity": color_identity,
                        },
                    )?;
                }
                if let Some(image_uris) = &card.image_uris {
                    tx.execute(
                        ADD_IMAGE_URIS_SQL,
                        named_params! {
                        ":card_id": &card.id,
                        ":small": image_uris.small,
                        ":normal": image_uris.normal,
                        ":large": image_uris.large,
                        ":png": image_uris.png,
                        ":art_crop": image_uris.art_crop,
                        ":border_crop": image_uris.border_crop
                        },
                    )?;
                }
                tx.commit()?;
                tx = conn.transaction()?;
            }
        }
        total_cards += group;
        println!("Inserted {} total cards", total_cards);
    }

    Ok(())
}

async fn download_json_from_api(path: &Path) -> Result<()> {
    let client = Client::new();
    let response = client
        .get("https://data.scryfall.io/default-cards/default-cards-20240220220632.json")
        .send()
        .await
        .context("Failed to send HTTP request")?;

    let mut file = tokio::fs::File::create(&path)
        .await
        .with_context(|| format!("Failed to create file: {:?}", &path))?;

    tokio::io::copy(&mut response.bytes().await?.as_ref(), &mut file)
        .await
        .context("Failed to write JSON data to file")?;

    Ok(())
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
