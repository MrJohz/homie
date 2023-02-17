-- SPDX-FileCopyrightText: 2023 Jonathan Frere
--
-- SPDX-License-Identifier: MPL-2.0
SELECT
  id,
  task_translations.task_name as name,
  kind,
  duration,
  participants,
  last_completed,
  last_completed_by
FROM
  grouped_tasks
  INNER JOIN task_translations ON task_translations.task_id = grouped_tasks.id
  AND task_translations.lang = ?
WHERE
  grouped_tasks.id = ?