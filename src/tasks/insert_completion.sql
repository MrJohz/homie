-- SPDX-FileCopyrightText: 2023 Jonathan Frere
--
-- SPDX-License-Identifier: MPL-2.0

INSERT INTO
  completions (task_id, completed_on, completed_by)
VALUES
  (
    (
      SELECT
        tasks.id
      from
        tasks
      WHERE
        tasks.task_name = ? COLLATE NOCASE
    ),
    ?,
    (
      SELECT
        users.id
      FROM
        users
      WHERE
        users.username = ? COLLATE NOCASE
    )
  )