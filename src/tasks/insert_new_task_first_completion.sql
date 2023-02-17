-- SPDX-FileCopyrightText: 2023 Jonathan Frere
--
-- SPDX-License-Identifier: MPL-2.0
INSERT INTO
  completions (task_id, completed_on, completed_by, initial)
SELECT
  ?,
  ?,
  id,
  TRUE
FROM
  users
WHERE
  users.username = ? COLLATE nocase
LIMIT
  1