use {
    serde::{Deserialize, Serialize},
    thiserror::Error,
};

fn default_as_false() -> bool {
    false
}

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
pub(crate) type Uri = String;
pub(crate) type CardFaces = Vec<CardFace>;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ImageUris {
    pub(crate) small: Option<Url>,
    pub(crate) normal: Option<Url>,
    pub(crate) large: Option<Url>,
    pub(crate) png: Option<Url>,
    pub(crate) art_crop: Option<Url>,
    pub(crate) border_crop: Option<Url>,
}

// Colors is WUBRG, etc.
type Colors = Vec<String>;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Card {
    // START CORE CARD FIELDS

    // This card’s Arena ID, if any. A large percentage of cards are not
    // available on Arena and do not have this ID.
    pub(crate) arena_id: Option<i64>,

    // A unique ID for this card in Scryfall’s database.
    pub(crate) id: Uuid,

    // A language code for this printing.
    pub(crate) lang: Option<String>,

    // This card’s Magic Online ID (also known as the Catalog ID), if any. A
    // large percentage of cards are not available on Magic Online and do not
    // have this ID.
    pub(crate) mtgo_id: Option<i64>,

    // This card’s foil Magic Online ID (also known as the Catalog ID), if any.
    // A large percentage of cards are not available on Magic Online and do not
    // have this ID.
    pub(crate) mtgo_foil_id: Option<i64>,

    // multiverse_ids	Array
    // Nullable
    // This card’s multiverse IDs on Gatherer, if any, as an array of integers. Note that Scryfall includes many promo cards, tokens, and other esoteric objects that do not have these identifiers.

    // This card’s ID on TCGplayer’s API, also known as the productId.
    pub(crate) tcgplayer_id: Option<i64>,

    // This card’s ID on TCGplayer’s API, for its etched version if that version
    // is a separate product.
    pub(crate) tcgplayer_etched_id: Option<i64>,

    // This card’s ID on Cardmarket’s API, also known as the idProduct.
    pub(crate) cardmarket_id: Option<i64>,

    // A content type for this object, always card.
    pub(crate) object: String,

    // A code for this card’s layout.
    pub(crate) layout: String,

    // A unique ID for this card’s oracle identity. This value is consistent
    // across reprinted card editions, and unique among different cards with the
    // same name (tokens, Unstable variants, etc). Always present except for the
    // reversible_card layout where it will be absent; oracle_id will be found
    // on each face instead.
    pub(crate) oracle_id: Option<Uuid>,

    // A link to where you can begin paginating all re/prints for this card on Scryfall’s API.
    pub(crate) prints_search_uri: Option<Url>,

    // A link to this card’s rulings list on Scryfall’s API.
    pub(crate) rulings_uri: Option<Url>,

    // A link to this card’s permapage on Scryfall’s website.
    pub(crate) scryfall_uri: Option<Url>,

    // END CORE CARD FIELDS

    // START GAMEPLAY FIELDS

    // all_parts	Array
    // Nullable
    // If this card is closely related to other cards, this property will be an array with Related Card Objects.

    // An array of Card Face objects, if this card is multifaced.
    pub(crate) card_faces: Option<CardFaces>,

    // The card’s mana value. Note that some funny cards have fractional mana
    // costs.
    pub(crate) cmc: Option<f64>,

    // This card’s color identity.
    pub(crate) color_identity: Colors,

    // The colors in this card’s color indicator, if any. A null value for this
    // field indicates the card does not have one.
    pub(crate) color_indicator: Option<Colors>,

    // This card’s colors, if the overall card has colors defined by the rules.
    // Otherwise the colors will be on the card_faces objects, see below.
    pub(crate) colors: Option<Colors>,

    // This face’s defense, if any.
    pub(crate) defense: Option<String>,

    // This card’s overall rank/popularity on EDHREC. Not all cards are ranked.
    pub(crate) edhrec_rank: Option<i64>,

    // This card’s hand modifier, if it is Vanguard card. This value will
    // contain a delta, such as -1.
    pub(crate) hand_modifier: Option<String>,

    // An array of keywords that this card uses, such as 'Flying' and 'Cumulative upkeep'.
    pub(crate) keywords: Vec<String>,

    // legalities	Object		An object describing the legality of this card across play formats. Possible legalities are legal, not_legal, restricted, and banned.
    // pub(crate) legalities: Object

    // This card’s life modifier, if it is Vanguard card. This value will
    // contain a delta, such as +2.
    pub(crate) life_modifier: Option<String>,

    // This loyalty if any. Note that some cards have loyalties that are not
    // numeric, such as X.
    pub(crate) loyalty: Option<String>,

    // The mana cost for this card. This value will be any empty string "" if
    // the cost is absent. Remember that per the game rules, a missing mana cost
    // and a mana cost of {0} are different values. Multi-faced cards will
    // report this value in card faces.
    pub(crate) mana_cost: Option<String>,

    // The name of this card. If this card has multiple faces, this field will
    // contain both names separated by ␣//␣.
    pub(crate) name: String,

    // The Oracle text for this card, if any.
    pub(crate) oracle_text: Option<String>,

    // This card’s rank/popularity on Penny Dreadful. Not all cards are ranked.
    pub(crate) penny_rank: Option<i64>,

    // This card’s power, if any. Note that some cards have powers that are not
    // numeric, such as *.
    pub(crate) power: Option<String>,

    // Colors of mana that this card could produce.
    pub(crate) produced_mana: Option<Colors>,

    // True if this card is on the Reserved List.
    pub(crate) reserved: bool,

    // This card’s toughness, if any. Note that some cards have toughnesses that
    // are not numeric, such as *.
    pub(crate) toughness: Option<String>,

    // The type line of this card.
    pub(crate) type_line: Option<String>,

    // END GAMEPLAY FIELDS

    // START PRINT FIELDS

    // The name of the illustrator of this card. Newly spoiled cards may not
    // have this field yet.
    pub(crate) artist: Option<String>,

    // // artist_ids	Array
    // // Nullable
    // // The IDs of the artists that illustrated this card. Newly spoiled cards may not have this field yet.
    // artist_ids: Artists

    // // attraction_lights	Array
    // // Nullable
    // // The lit Unfinity attractions lights on this card, if any.
    // attraction_light: AttractionLights

    // Whether this card is found in boosters.
    pub(crate) booster: bool,

    // This card’s border color: black, white, borderless, silver, or gold.
    pub(crate) border_color: String,

    // The Scryfall ID for the card back design present on this card.
    pub(crate) card_back_id: Option<Uuid>,

    // This card’s collector number. Note that collector numbers can contain
    // non-numeric characters, such as letters or ★.
    pub(crate) collector_number: String,

    // True if you should consider avoiding use of this print downstream.
    #[serde(default = "default_as_false")]
    pub(crate) content_warning: bool,

    // True if this card was only released in a video game.
    pub(crate) digital: bool,

    // An array of computer-readable flags that indicate if this card can come in foil, nonfoil, or etched finishes.
    pub(crate) finishes: Vec<String>,

    // The just-for-fun name printed on the card (such as for Godzilla series
    // cards).
    pub(crate) flavor_name: Option<String>,

    // The flavor text, if any.
    pub(crate) flavor_text: Option<String>,

    // // frame_effects	Array
    // // Nullable
    // // This card’s frame effects, if any.
    // frame_effects: FrameEffects,

    // This card’s frame layout.
    pub(crate) frame: String,

    // True if this card’s artwork is larger than normal.
    pub(crate) full_art: bool,

    // A list of games that this card print is available in, paper, arena, and/or mtgo.
    pub(crate) games: Vec<String>,

    // True if this card’s imagery is high resolution.
    pub(crate) highres_image: bool,

    // A unique identifier for the card artwork that remains consistent across
    // reprints. Newly spoiled cards may not have this field yet.
    pub(crate) illustration_id: Option<Uuid>,

    // A computer-readable indicator for the state of this card’s image, one of
    // missing, placeholder, lowres, or highres_scan.
    pub(crate) image_status: String,

    // An object listing available imagery for this card. See the Card Imagery
    // article for more information.
    pub(crate) image_uris: Option<ImageUris>,

    // True if this card is oversized.
    pub(crate) oversized: bool,

    // // prices	Object		An object containing daily price information for this card, including usd, usd_foil, usd_etched, eur, eur_foil, eur_etched, and tix prices, as strings.
    // prices: Prices

    // The localized name printed on this card, if any.
    pub(crate) printed_name: Option<String>,

    // The localized text printed on this card, if any.
    pub(crate) printed_text: Option<String>,

    // The localized type line printed on this card, if any.
    pub(crate) printed_type_line: Option<String>,

    // True if this card is a promotional print.
    pub(crate) promo: bool,

    // An array of strings describing what categories of promo cards this card
    // falls into.
    #[serde(default = "Vec::new")]
    pub(crate) promo_types: Vec<String>,

    // // purchase_uris	Object
    // // Nullable
    // // An object providing URIs to this card’s listing on major marketplaces. Omitted if the card is unpurchaseable.
    // purchase_uris: Option<PurchaseURIs>,

    // This card’s rarity. One of common, uncommon, rare, special, mythic, or bonus.
    pub(crate) rarity: String,

    // // related_uris	Object		An object providing URIs to this card’s listing on other Magic: The Gathering online resources.
    // related_uris: Option<RelatedUris>,

    // TODO - I should probably parse this into a date object instead of string.
    // The date this card was first released.
    pub(crate) released_at: String,

    // True if this card is a reprint.
    pub(crate) reprint: bool,

    // A link to this card’s set on Scryfall’s website.
    pub(crate) scryfall_set_uri: Uri,

    // This card’s full set name.
    pub(crate) set_name: String,

    // A link to where you can begin paginating this card’s set on the Scryfall
    // API.
    pub(crate) set_search_uri: Uri,

    // The type of set this printing is in.
    pub(crate) set_type: String,

    // A link to this card’s set object on Scryfall’s API.
    pub(crate) set_uri: Uri,

    // This card’s set code.
    pub(crate) set: String,

    // This card’s Set object UUID.
    pub(crate) set_id: Uuid,

    // True if this card is a Story Spotlight.
    pub(crate) story_spotlight: bool,

    // True if the card is printed without text.
    pub(crate) textless: bool,

    // Whether this card is a variation of another printing.
    pub(crate) variation: bool,

    // The printing ID of the printing this card is a variation of.
    pub(crate) variation_of: Option<Uuid>,

    // The security stamp on this card, if any. One of oval, triangle, acorn,
    // circle, arena, or heart.
    pub(crate) security_stamp: Option<String>,

    // This card’s watermark, if any.
    pub(crate) watermark: Option<String>,
    // // preview.previewed_at	Date
    // // Nullable
    // // The date this card was previewed.
    // // preview.source_uri	URI
    // // Nullable
    // // A link to the preview for this card.
    // // preview.source	String
    // // Nullable
    // // The name of the source that previewed this card.
    // preview: Option<Preview>,

    // END PRINT FIELDS
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CardFace {
    // The name of the illustrator of this card face. Newly spoiled cards may
    // not have this field yet.
    pub(crate) artist: Option<String>,
    // The ID of the illustrator of this card face. Newly spoiled cards may not
    // have this field yet.
    pub(crate) artist_id: Option<Uuid>,
    // The mana value of this particular face, if the card is reversible.
    pub(crate) cmc: Option<f64>,
    // The colors in this face’s color indicator, if any.
    pub(crate) color_indicator: Option<Colors>,
    // This face’s colors, if the game defines colors for the individual face of
    // this card.
    pub(crate) colors: Option<Colors>,
    // This face’s defense, if the game defines colors for the individual face
    // of this card.
    pub(crate) defense: Option<String>,
    // The flavor text printed on this face, if any.
    pub(crate) flavor_text: Option<String>,
    // A unique identifier for the card face artwork that remains consistent
    // across reprints. Newly spoiled cards may not have this field yet.
    pub(crate) illustration_id: Option<Uuid>,
    // An object providing URIs to imagery for this face, if this is a
    // double-sided card. If this card is not double-sided, then the image_uris
    // property will be part of the parent object instead.
    pub(crate) image_uris: Option<ImageUris>,
    // The layout of this card face, if the card is reversible.
    pub(crate) layout: Option<String>,
    // This face’s loyalty, if any.
    pub(crate) loyalty: Option<String>,
    // The mana cost for this face. This value will be any empty string "" if
    // the cost is absent. Remember that per the game rules, a missing mana cost
    // and a mana cost of {0} are different values.
    pub(crate) mana_cost: Option<String>,
    // The name of this particular face.
    pub(crate) name: String,
    // A content type for this object, always card_face.
    pub(crate) object: String,
    // The Oracle ID of this particular face, if the card is reversible.
    pub(crate) oracle_id: Option<Uuid>,
    // The Oracle text for this face, if any.
    pub(crate) oracle_text: Option<String>,
    // This face’s power, if any. Note that some cards have powers that are not
    // numeric, such as *.
    pub(crate) power: Option<String>,
    // The localized name printed on this face, if any.
    pub(crate) printed_name: Option<String>,
    // The localized text printed on this face, if any.
    pub(crate) printed_text: Option<String>,
    // The localized type line printed on this face, if any.
    pub(crate) printed_type_line: Option<String>,
    // This face’s toughness, if any.
    pub(crate) toughness: Option<String>,
    // The type line of this particular face, if the card is reversible.
    pub(crate) type_line: Option<String>,
    // The watermark on this particulary card face, if any.
    pub(crate) watermark: Option<String>,
}
