CREATE TABLE
  tasks (
    id text primary key,
    task_name text NOT NULL,
    kind text NOT NULL,
    duration integer NOT NULL,
    start_assignee text NOT NULL,
    first_done text NOT NULL
  );

CREATE TABLE
  task_participant_link (
    task_id text NOT NULL REFERENCES tasks (id),
    user_id text NOT NULL REFERENCES users (id)
  );

CREATE TABLE
  completions (
    task_id text NOT NULL REFERENCES tasks (id),
    completed_by text NOT NULL REFERENCES users (id),
    completed_on text NOT NULL
  );