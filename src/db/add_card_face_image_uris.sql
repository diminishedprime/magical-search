INSERT OR REPLACE INTO card_faces_image_uris (
    card_id,
    face_index,
    small,
    normal,
    large,
    png,
    art_crop,
    border_crop
) VALUES (
    :card_id,
    :face_index,
    :small,
    :normal,
    :large,
    :png,
    :art_crop,
    :border_crop
);
