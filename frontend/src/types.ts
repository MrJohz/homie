// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

export type TaskId = number & { __task_id_brand: "TASK_ID_BRAND" };

export type ITask = {
  id: TaskId;
  name: string;
  kind: "Schedule" | "Interval";
  assigned_to: string;
  deadline: { Overdue: number } | { Upcoming: number };
  length_days: number;
  last_completed: string;
  participants: string[];
};
