SELECT 
    c.id,
    c.name,
    c.cmc,
    ciu.small AS small_image_url,
    cib.small AS small_image_blob
FROM 
    cards c
LEFT JOIN 
    card_image_uris ciu ON c.id = ciu.card_id
LEFT JOIN 
    card_image_blobs cib ON c.id = cib.card_id
WHERE
    c.id = :id;
