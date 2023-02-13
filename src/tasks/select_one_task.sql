-- SPDX-FileCopyrightText: 2023 Jonathan Frere
--
-- SPDX-License-Identifier: MPL-2.0

SELECT
  id,
  name,
  kind,
  duration,
  participants,
  last_completed,
  last_completed_by
FROM
  grouped_tasks
WHERE
  grouped_tasks.name = ? COLLATE NOCASE