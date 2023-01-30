INSERT INTO
  task_participant_link (task_id, user_id)
SELECT
  ?,
  id
FROM
  users
WHERE
  users.username = ? COLLATE nocase