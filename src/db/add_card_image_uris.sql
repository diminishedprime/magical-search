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