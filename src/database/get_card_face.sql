SELECT
    uris.small,
    uris.normal,
    uris.large,
    faces.image
FROM
    card_faces AS faces
LEFT JOIN
    card_faces_image_uris AS uris ON faces.card_id = uris.card_id AND faces.face_index = uris.face_index
WHERE
    faces.card_id = :card_id AND 
    faces.face_index = :face_index;
