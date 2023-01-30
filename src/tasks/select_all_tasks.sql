SELECT
  tasks.id as id,
  tasks.task_name as name,
  tasks.kind as kind,
  tasks.duration as duration,
  json_group_array (p1.username) as participants,
  CASE tasks.kind
    WHEN "Interval" THEN last_completion.completed_on
    WHEN "Schedule" THEN date (
      first_completion.completed_on,
      '+' || (tasks.duration * coalesce(completion_count, 1)) || ' days'
    )
    ELSE NULL
  END as last_completed,
  u_completed.username as last_completed_by,
  count(p1.ordering) as _ignore_me
FROM
  tasks
  INNER JOIN (
    SELECT
      _p.rowid as ordering,
      task_id,
      _u.username
    FROM
      task_participant_link _p
      INNER JOIN users _u ON _u.id = _p.user_id
    ORDER BY
      _p.rowid
  ) p1 ON p1.task_id = tasks.id
  INNER JOIN completions last_completion ON tasks.id = last_completion.task_id
  AND last_completion.rowid = (
    SELECT
      c2.rowid
    FROM
      completions AS c2
    WHERE
      c2.task_id = tasks.id
    ORDER BY
      c2.completed_on DESC,
      c2.rowid DESC
    LIMIT
      1
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
GROUP BY
  tasks.id