SELECT
  id,
  name,
  kind,
  duration,
  participants,
  last_completed,
  last_completed_by
FROM
  grouped_tasks
WHERE
  grouped_tasks.name = ? COLLATE NOCASE