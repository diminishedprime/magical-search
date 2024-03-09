UPDATE card_faces
SET image = :image
WHERE 
    card_id = :card_id AND 
    face_index = :face_index;
