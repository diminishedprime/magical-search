mod db;
mod types;

use anyhow::{Context, Result};
use db::{ADD_CARD_FACE, ADD_CARD_FACE_COLORS, ADD_CARD_FACE_IMAGE_URIS, CREATE_TABLE_SQL};
use itertools::Itertools;
use reqwest::Client;
use rusqlite::{named_params, Transaction};
use serde::de::DeserializeOwned;
use serde_json::{self, Deserializer};
use std::fs::File;
use std::io::BufReader;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use types::{Card, CardFace};

use crate::db::{
    ADD_CARD, ADD_CARD_COLORS, ADD_CARD_COLOR_IDENTITY, ADD_CARD_IMAGE_URIS, ADD_CARD_KEYWORDS,
};

static CARD_CHUNK_SIZE: usize = 1000;

fn add_card(tx: &mut Transaction, card: &Card) -> Result<()> {
    add_base_card(tx, &card)?;
    add_card_faces(tx, &card)?;
    add_colors(tx, &card)?;
    add_keywords(tx, &card)?;
    add_color_identity(tx, &card)?;
    add_image_uris(tx, &card)?;
    Ok(())
}

fn add_base_card(tx: &mut Transaction, card: &Card) -> Result<()> {
    tx.execute(
        ADD_CARD,
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
        ":cmc": card.cmc,
        ":power": card.power,
        ":toughness": card.toughness,
        ":flavor_text": card.flavor_text,
        ":oracle_text": card.oracle_text,
        },
    )?;
    Ok(())
}

fn add_base_card_face(
    tx: &mut Transaction,
    card: &Card,
    card_face: &CardFace,
    idx: usize,
) -> Result<()> {
    tx.execute(
        ADD_CARD_FACE,
        named_params! {
            ":face_index": idx,
            ":card_id": &card.id,
            ":artist": &card_face.artist,
            ":artist_id": &card_face.artist_id,
            ":cmc": &card_face.cmc,
            ":defense": &card_face.defense,
            ":flavor_text": &card_face.flavor_text,
            ":illustration_id": &card_face.illustration_id,
            ":layout": &card_face.layout,
            ":loyalty": &card_face.loyalty,
            ":mana_cost": &card_face.mana_cost,
            ":name": &card_face.name,
            ":oracle_id": &card_face.oracle_id,
            ":oracle_text": &card_face.oracle_text,
            ":power": &card_face.power,
            ":printed_name": &card_face.printed_name,
            ":printed_text": &card_face.printed_text,
            ":printed_type_line": &card_face.printed_type_line,
            ":toughness": &card_face.toughness,
            ":type_line": &card_face.type_line,
            ":watermark": &card_face.watermark,
        },
    )?;
    Ok(())
}

fn add_card_faces(tx: &mut Transaction, card: &Card) -> Result<()> {
    if let Some(card_faces) = &card.card_faces {
        for (idx, card_face) in card_faces.iter().enumerate() {
            add_base_card_face(tx, card, card_face, idx)?;
            add_card_face_colors(tx, card, card_face, idx)?;
            // add_card_face_color_identity(tx, card)?;
            add_card_face_image_uris(tx, card, card_face, idx)?;
        }
    }
    Ok(())
}

fn add_card_face_image_uris(
    tx: &mut Transaction<'_>,
    card: &Card,
    card_face: &CardFace,
    idx: usize,
) -> Result<()> {
    if let Some(image_uris) = &card_face.image_uris {
        tx.execute(
            ADD_CARD_FACE_IMAGE_URIS,
            named_params! {
                ":face_index": idx,
                ":card_id": &card.id,
                ":small": image_uris.small,
                ":normal": image_uris.normal,
                ":large": image_uris.large,
                ":png": image_uris.png,
                ":art_crop": image_uris.art_crop,
                ":border_crop": image_uris.border_crop,
            },
        )?;
    }
    Ok(())
}

fn add_card_face_colors(
    tx: &mut Transaction<'_>,
    card: &Card,
    card_face: &CardFace,
    idx: usize,
) -> Result<()> {
    if let Some(colors) = &card_face.colors {
        for color in colors {
            tx.execute(
                ADD_CARD_FACE_COLORS,
                named_params! {
                    ":face_index": idx,
                    ":card_id": card.id,
                    ":color": color,
                },
            )?;
        }
    }
    Ok(())
}

fn add_colors(tx: &mut Transaction, card: &Card) -> Result<()> {
    if let Some(colors) = &card.colors {
        for color in colors {
            tx.execute(
                ADD_CARD_COLORS,
                named_params! {
                    ":card_id": &card.id,
                    ":color": color,
                },
            )?;
        }
    }
    Ok(())
}

fn add_keywords(tx: &mut Transaction, card: &Card) -> Result<()> {
    for keyword in &card.keywords {
        tx.execute(
            ADD_CARD_KEYWORDS,
            named_params! {
                ":card_id": &card.id,
                ":keyword": keyword,
            },
        )?;
    }
    Ok(())
}

fn add_color_identity(tx: &mut Transaction, card: &Card) -> Result<()> {
    for color_identity in &card.color_identity {
        tx.execute(
            ADD_CARD_COLOR_IDENTITY,
            named_params! {
                ":card_id": &card.id,
                ":color_identity": color_identity,
            },
        )?;
    }
    Ok(())
}

fn add_image_uris(tx: &mut Transaction, card: &Card) -> Result<()> {
    if let Some(image_uris) = &card.image_uris {
        tx.execute(
            ADD_CARD_IMAGE_URIS,
            named_params! {
                ":card_id": &card.id,
                ":small": image_uris.small,
                ":normal": image_uris.normal,
                ":large": image_uris.large,
                ":png": image_uris.png,
                ":art_crop": image_uris.art_crop,
                ":border_crop": image_uris.border_crop,
            },
        )?;
    }
    Ok(())
}

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
                add_card(&mut tx, &card)?;
            }
            tx.commit()?;
            tx = conn.transaction()?;
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
