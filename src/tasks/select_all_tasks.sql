SELECT
  tasks.task_name as name,
  tasks.kind as kind,
  tasks.duration as duration,
  u_participants.username as participant,
  tasks.first_done as first_done,
  IFNULL (completions.completed_on, tasks.first_done) as done_at,
  IFNULL (u_completed.username, u_assignee.username) as done_by
FROM
  tasks
  INNER JOIN task_participant_link ON tasks.id = task_participant_link.task_id
  INNER JOIN users u_participants ON u_participants.id = task_participant_link.user_id
  INNER JOIN users u_assignee ON u_assignee.id = tasks.start_assignee
  LEFT JOIN completions ON tasks.id = completions.task_id
  AND completions.rowid = (
    Select
      Max(rowid)
    from
      completions as c2
    where
      c2.task_id = tasks.id
  )
  LEFT JOIN users u_completed ON u_completed.id = completions.completed_by
ORDER BY
  tasks.task_name ASC,
  task_participant_link.rowid ASC