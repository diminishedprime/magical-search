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
    cmc DECIMAL(32,16),
    power TEXT,
    toughness TEXT,
    flavor_text TEXT,
    oracle_text TEXT
);

CREATE TABLE IF NOT EXISTS card_faces (
    face_index INTEGER NOT NULL,
    card_id TEXT,
    artist TEXT,
    artist_id TEXT,
    cmc DECIMAL(32,16),
    defense TEXT,
    flavor_text TEXT,
    illustration_id TEXT,
    layout TEXT,
    loyalty TEXT,
    mana_cost TEXT,
    name TEXT,
    oracle_id TEXT,
    oracle_text TEXT,
    power TEXT,
    printed_name TEXT,
    printed_text TEXT,
    printed_type_line TEXT,
    toughness TEXT,
    type_line TEXT,
    watermark TEXT,
    PRIMARY KEY (card_id, face_index),
    FOREIGN KEY (card_id) REFERENCES cards(id)
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

CREATE TABLE IF NOT EXISTS card_image_blobs (
    card_id TEXT,
    small BLOB,
    normal BLOB,
    large BLOB,
    png BLOB,
    art_crop BLOB,
    border_crop BLOB,
    FOREIGN KEY (card_id) REFERENCES cards(id),
    CONSTRAINT unique_blob_card_id UNIQUE (card_id, small, normal, large, png, art_crop, border_crop)
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

CREATE TABLE IF NOT EXISTS card_faces_colors (
    card_id TEXT,
    face_index INTEGER,
    color TEXT,
    FOREIGN KEY (card_id, face_index) REFERENCES card_faces(card_id, face_index)
);

CREATE TABLE IF NOT EXISTS card_faces_image_uris (
    card_id TEXT,
    face_index INTEGER,
    small TEXT,
    normal TEXT,
    large TEXT,
    png TEXT,
    art_crop TEXT,
    border_crop TEXT,
    FOREIGN KEY (card_id, face_index) REFERENCES card_faces(card_id, face_index)
);

CREATE TABLE IF NOT EXISTS card_faces_image_blobs (
    card_id TEXT,
    face_index INTEGER,
    small BLOB,
    normal BLOB,
    large BLOB,
    png BLOB,
    art_crop BLOB,
    border_crop BLOB,
    FOREIGN KEY (card_id, face_index) REFERENCES card_faces(card_id, face_index)
);