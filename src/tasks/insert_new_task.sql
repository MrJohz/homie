-- SPDX-FileCopyrightText: 2023 Jonathan Frere
--
-- SPDX-License-Identifier: MPL-2.0

INSERT INTO
  tasks (task_name, kind, duration)
VALUES
  (?, ?, ?) RETURNING tasks.id