CREATE TABLE
  tasks (
    id integer primary key autoincrement,
    task_name text NOT NULL,
    kind text NOT NULL,
    duration integer NOT NULL,
    UNIQUE (task_name)
  );

CREATE TABLE
  task_participant_link (
    task_id integer NOT NULL REFERENCES tasks (id),
    user_id integer NOT NULL REFERENCES users (id),
    UNIQUE (task_id, user_id)
  );

CREATE TABLE
  completions (
    task_id integer NOT NULL REFERENCES tasks (id),
    completed_by integer NOT NULL REFERENCES users (id),
    completed_on text NOT NULL,
    initial integer NOT NULL DEFAULT FALSE
  );

CREATE INDEX task_participant_link_task_id ON task_participant_link (task_id);

CREATE INDEX completions_completed_on_task_id ON completions (completed_on, task_id);

CREATE VIEW
  grouped_tasks AS
WITH
  participants AS (
    SELECT
      _p.rowid as ordering,
      task_id,
      _u.username
    FROM
      task_participant_link _p
      INNER JOIN users _u ON _u.id = _p.user_id
    ORDER BY
      _p.rowid
  )
SELECT
  tasks.id as id,
  tasks.task_name as name,
  tasks.kind as kind,
  tasks.duration as duration,
  json_group_array (participants.username) as participants,
  CASE tasks.kind
    WHEN "Interval" THEN last_completion.completed_on
    WHEN "Schedule" THEN date (
      first_completion.completed_on,
      '+' || (tasks.duration * coalesce(completion_count, 0)) || ' days'
    )
    ELSE NULL
  END as last_completed,
  u_completed.username as last_completed_by,
  count(participants.ordering) as _ignore_me
FROM
  tasks
  INNER JOIN participants ON participants.task_id = tasks.id
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
  tasks.id;