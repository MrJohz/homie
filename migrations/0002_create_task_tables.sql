CREATE TABLE tasks (
    id text primary key,
    task_name text,
    kind text,
    duration integer
);

CREATE TABLE task_user_link (
  task_id text REFERENCES tasks (id),
  user_id text REFERENCES users (id)
);

CREATE TABLE completions (
  task_id text REFERENCES tasks (id),
  completed_by text REFERENCES users (id),
  completed_year integer,
  completed_month integer,
  completed_day integer
);
