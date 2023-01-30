INSERT INTO
  tasks (task_name, kind, duration)
VALUES
  (?, ?, ?) RETURNING tasks.id