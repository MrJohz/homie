-- SPDX-FileCopyrightText: 2023 Jonathan Frere
--
-- SPDX-License-Identifier: MPL-2.0
BEGIN TRANSACTION;

INSERT INTO
  tasks (kind, duration)
VALUES
  (?, ?) RETURNING tasks.id;

INSERT INTO
  task_translations (task_id, task_name)
VALUES
  (last_insert_rowid (), ?);

COMMIT TRANSACTION;