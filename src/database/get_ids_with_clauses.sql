SELECT cards.id
FROM cards
{clauses}
LIMIT :limit
OFFSET :cursor;