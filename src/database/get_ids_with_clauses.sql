SELECT MIN(cards.id) as id
FROM cards
{joins}
{clauses}
GROUP BY cards.name
LIMIT :limit
OFFSET :cursor;