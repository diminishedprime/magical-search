SELECT 
    c.id,
    c.name,
    c.cmc,
    ciu.small AS small_image_url,
    ciu.normal AS normal_image_url,
    ciu.large AS large_image_url,
    image,
    (SELECT COUNT(*) FROM card_faces_image_uris WHERE card_id = c.id) AS num_faces,
    c.oracle_text
FROM 
    cards c
LEFT JOIN 
    card_image_uris ciu ON c.id = ciu.card_id
WHERE
    c.id = :id;
