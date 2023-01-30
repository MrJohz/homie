SELECT
  tasks.task_name as name,
  tasks.kind as kind,
  tasks.duration as duration,
  json_group_array (u_participants.username) as participants,
  CASE tasks.kind
    WHEN "INTERVAL" THEN last_completion.completed_on
    WHEN "SCHEDULE" THEN date (
      first_completion.completed_on,
      '+' || (tasks.duration * coalesce(completion_count, 0)) || ' days'
    )
    ELSE NULL
  END as last_completed,
  u_completed.username as last_completed_by
FROM
  tasks
  INNER JOIN task_participant_link ON tasks.id = task_participant_link.task_id
  INNER JOIN users u_participants ON u_participants.id = task_participant_link.user_id
  INNER JOIN completions last_completion ON tasks.id = last_completion.task_id
  AND last_completion.completed_on = (
    Select
      max(completed_on)
    from
      completions as c2
    where
      c2.task_id = tasks.id
  )
  INNER JOIN users u_completed ON u_completed.id = last_completion.completed_by
  INNER JOIN completions first_completion ON tasks.id = first_completion.task_id
  AND first_completion.completed_on = (
    Select
      max(completed_on)
    from
      completions as c3
    where
      c3.task_id = tasks.id
      AND c3.initial = TRUE
  )
  LEFT JOIN (
    select
      task_id,
      count(*) as completion_count
    FROM
      completions _ccount
    WHERE
      _ccount.initial = FALSE
  ) c4 ON c4.task_id = tasks.id
WHERE
  tasks.id = ?