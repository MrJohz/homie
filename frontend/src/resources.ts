import { endOfYesterday, startOfTomorrow, sub } from "date-fns";
import { ITask } from "./types";

export type Success<T> = { k: "ok"; value: T };
export type Error<T> = { k: "err"; value: T };
export type Result<T, U> = Success<T> | Error<U>;

export async function createToken(
  username: string,
  password: string
): Promise<Success<string> | Error<"BAD_CONNECTION" | "BAD_AUTH">> {
  const response = await fetch("/api/auth/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  });

  if (!response.ok) {
    return {
      k: "err",
      value: response.status === 0 ? "BAD_CONNECTION" : "BAD_AUTH",
    };
  }

  return { k: "ok", value: await response.json() };
}

export async function fetchTasks(args: {
  token: string;
}): Promise<Result<ITask[], "BAD_AUTH" | "BAD_CONNECTION">> {
  const response = await fetch("/api/tasks", {
    headers: { token: args.token },
  });

  if (!response.ok) {
    return {
      k: "err",
      value: response.status === 0 ? "BAD_CONNECTION" : "BAD_AUTH",
    };
  }

  const tasks = await response.json();

  return {
    k: "ok",
    value: [
      ...tasks,
      {
        name: "Test Task Tomorrow",
        assigned_to: "Jonathan",
        deadline: { Upcoming: 1 },
        kind: "Interval",
        last_completed: new Date(),
        length_days: 1,
      },
      {
        name: "Test Task Today",
        assigned_to: "Jonathan",
        deadline: { Upcoming: 0 },
        kind: "Interval",
        last_completed: endOfYesterday(),
        length_days: 1,
      },
      {
        name: "Test Task Yesterday",
        assigned_to: "Jonathan",
        deadline: { Overdue: 1 },
        kind: "Interval",
        last_completed: sub(new Date(), { days: 2 }),
        length_days: 1,
      },
      {
        name: "Test Task Tomorrow",
        assigned_to: "Jonathan",
        deadline: { Upcoming: 1 },
        kind: "Schedule",
        last_completed: new Date(),
        length_days: 1,
      },
      {
        name: "Test Task Today",
        assigned_to: "Jonathan",
        deadline: { Upcoming: 0 },
        kind: "Schedule",
        last_completed: endOfYesterday(),
        length_days: 1,
      },
      {
        name: "Test Task Yesterday",
        assigned_to: "Jonathan",
        deadline: { Overdue: 1 },
        kind: "Schedule",
        last_completed: sub(new Date(), { days: 2 }),
        length_days: 1,
      },
    ],
  };
}
