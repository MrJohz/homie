export type ITask = {
  name: string;
  kind: "Schedule" | "Interval";
  assigned_to: string;
  deadline: { Overdue: number } | { Upcoming: number };
};
