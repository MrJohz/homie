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

  return { k: "ok", value: await response.json() };
}
