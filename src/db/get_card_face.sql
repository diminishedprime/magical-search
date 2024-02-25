SELECT
    ciu.small AS small_image_url,
    cib.small AS small_image_blob
FROM 
    card_faces cf
LEFT JOIN 
    card_faces_image_uris ciu ON cf.card_id = ciu.card_id
                        AND cf.face_index = ciu.face_index
LEFT JOIN 
    card_faces_image_blobs cib ON cf.card_id = cib.card_id
                        AND cf.face_index = cib.face_index
WHERE
    cf.card_id = :card_id
    AND cf.face_index = :face_index;
