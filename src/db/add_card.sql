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
    cmc,
    power,
    toughness,
    flavor_text,
    oracle_text,
    C,
    W,
    U,
    B,
    R,
    G,
    type_line
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
    :cmc,
    :power,
    :toughness,
    :flavor_text,
    :oracle_text,
    :C,
    :W,
    :U,
    :B,
    :R,
    :G,
    :type_line
)
