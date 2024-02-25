INSERT OR REPLACE INTO card_faces_image_blobs (
    card_id,
    face_index,
    small
) VALUES (
    :card_id,
    :face_index,
    :small_blob
);
