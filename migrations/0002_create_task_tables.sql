CREATE TABLE
  tasks (
    id integer primary key,
    task_name text NOT NULL,
    kind text NOT NULL,
    duration integer NOT NULL
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