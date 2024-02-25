SELECT 
    c.id,
    c.name,
    c.cmc,
    ciu.small AS small_image_url,
    cib.small AS small_image_blob,
    (SELECT COUNT(*) FROM card_faces_image_uris WHERE card_id = c.id) AS num_faces
FROM 
    cards c
LEFT JOIN 
    card_image_uris ciu ON c.id = ciu.card_id
LEFT JOIN 
    card_image_blobs cib ON c.id = cib.card_id
WHERE
    c.id = :id;