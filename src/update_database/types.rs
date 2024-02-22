use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum BuildError {
    #[error("Failed to open JSON file: {0}")]
    JsonFileError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Failed to open SQLite database: {0}")]
    DatabaseError(#[from] rusqlite::Error),
}

pub(crate) type Uuid = String;
pub(crate) type Url = String;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ImageUris {
    pub(crate) small: Option<Url>,
    pub(crate) normal: Option<Url>,
    pub(crate) large: Option<Url>,
    pub(crate) png: Option<Url>,
    pub(crate) art_crop: Option<Url>,
    pub(crate) border_crop: Option<Url>,
}

// Colors is WUBRG
type Colors = Vec<String>;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Card {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) lang: Option<String>,
    pub(crate) object: String,
    pub(crate) layout: String,
    pub(crate) arena_id: Option<i64>,
    pub(crate) mtgo_id: Option<i64>,
    pub(crate) mtgo_foil_id: Option<i64>,
    pub(crate) tcgplayer_id: Option<i64>,
    pub(crate) tcgplayer_etched_id: Option<i64>,
    pub(crate) cardmarket_id: Option<i64>,
    pub(crate) oracle_id: Option<Uuid>,
    pub(crate) prints_search_uri: Option<Url>,
    pub(crate) rulings_uri: Option<Url>,
    pub(crate) scryfall_uri: Option<Url>,
    pub(crate) uri: Option<Url>,
    pub(crate) cmc: Option<f64>,
    pub(crate) image_uris: Option<ImageUris>,
    pub(crate) color_identity: Colors,
    pub(crate) color_indicator: Option<Colors>,
    pub(crate) colors: Option<Colors>,
    pub(crate) defense: Option<String>,
    pub(crate) edhrec_rank: Option<i64>,
    pub(crate) hand_modifier: Option<String>,
    pub(crate) keywords: Vec<String>,
    // pub(crate) legalities: Object
    pub(crate) life_modifier: Option<String>,
    pub(crate) loyalty: Option<String>,
    pub(crate) mana_cost: Option<String>,
    pub(crate) oracle_text: Option<String>,
    pub(crate) penny_rank: Option<i64>,
    pub(crate) power: Option<String>,
    pub(crate) produced_mana: Option<Colors>,
    pub(crate) reserved: bool,
    pub(crate) toughness: Option<String>,
    pub(crate) type_line: Option<String>,
    pub(crate) flavor_text: Option<String>,
}
