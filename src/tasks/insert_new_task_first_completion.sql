INSERT INTO
  completions (task_id, completed_on, completed_by, initial)
SELECT
  ?,
  ?,
  id,
  TRUE
FROM
  users
WHERE
  users.username = ? COLLATE nocase
LIMIT
  1