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
    // A code for this card’s layout. See https://scryfall.com/docs/api/layouts
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
    pub(crate) card_faces: Option<Vec<CardFace>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CardFace {
    // The name of the illustrator of this card face. Newly spoiled cards may not have this field yet.
    pub(crate) artist: Option<String>,
    // The ID of the illustrator of this card face. Newly spoiled cards may not have this field yet.
    pub(crate) artist_id: Option<Uuid>,
    // The mana value of this particular face, if the card is reversible.
    pub(crate) cmc: Option<f64>,
    // The colors in this face’s color indicator, if any.
    pub(crate) color_indicator: Option<Colors>,
    // This face’s colors, if the game defines colors for the individual face of this card.
    pub(crate) colors: Option<Colors>,
    // This face’s defense, if the game defines colors for the individual face of this card.
    pub(crate) defense: Option<String>,
    // The flavor text printed on this face, if any.
    pub(crate) flavor_text: Option<String>,
    // A unique identifier for the card face artwork that remains consistent across reprints. Newly spoiled cards may not have this field yet.
    pub(crate) illustration_id: Option<Uuid>,
    // An object providing URIs to imagery for this face, if this is a
    // double-sided card. If this card is not double-sided, then the image_uris
    // property will be part of the parent object instead.
    image_uris: Option<ImageUris>,
    // layout	String
    // Nullable
    // The layout of this card face, if the card is reversible.
    // loyalty	String
    // Nullable
    // This face’s loyalty, if any.
    // mana_cost	String		The mana cost for this face. This value will be any empty string "" if the cost is absent. Remember that per the game rules, a missing mana cost and a mana cost of {0} are different values.
    // name	String		The name of this particular face.
    // object	String		A content type for this object, always card_face.
    // oracle_id	UUID
    // Nullable
    // The Oracle ID of this particular face, if the card is reversible.
    // oracle_text	String
    // Nullable
    // The Oracle text for this face, if any.
    // power	String
    // Nullable
    // This face’s power, if any. Note that some cards have powers that are not numeric, such as *.
    // printed_name	String
    // Nullable
    // The localized name printed on this face, if any.
    // printed_text	String
    // Nullable
    // The localized text printed on this face, if any.
    // printed_type_line	String
    // Nullable
    // The localized type line printed on this face, if any.
    // toughness	String
    // Nullable
    // This face’s toughness, if any.
    // type_line	String
    // Nullable
    // The type line of this particular face, if the card is reversible.
    // watermark	String
    // Nullable
    // The watermark on this particulary card face, if any.
}
