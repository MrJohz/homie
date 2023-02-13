// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

export type ITask = {
  name: string;
  kind: "Schedule" | "Interval";
  assigned_to: string;
  deadline: { Overdue: number } | { Upcoming: number };
  length_days: number;
  last_completed: string;
  participants: string[];
};
