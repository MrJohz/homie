import { endOfYesterday, startOfTomorrow, sub } from "date-fns";
import { ITask } from "./types";

export type Success<T> = { k: "ok"; value: T };
export type Error<T> = { k: "err"; value: T };
export type Result<T, U> = Success<T> | Error<U>;

type ExtendedRequestInit = Omit<RequestInit, "body"> & {
  body?: any;
};

async function fetchWrapper(
  url: string,
  details: ExtendedRequestInit
): Promise<Result<Response, "BAD_CONNECTION">> {
  try {
    const response = await fetch(url, {
      ...details,
      headers: {
        ...details.headers,
        "Content-Type": "application/json",
      },
      body: details.body == null ? undefined : JSON.stringify(details.body),
    });
    return { k: "ok", value: response };
  } catch (e) {
    return { k: "err", value: "BAD_CONNECTION" };
  }
}

export async function createToken(
  username: string,
  password: string
): Promise<Success<string> | Error<"BAD_CONNECTION" | "BAD_AUTH">> {
  const response = await fetchWrapper("/api/auth/login", {
    method: "POST",
    body: { username, password },
  });
  if (response.k === "err") return response;

  if (!response.value.ok) {
    return {
      k: "err",
      value: "BAD_AUTH",
    };
  }

  return { k: "ok", value: await response.value.json() };
}

export async function fetchTasks(args: {
  token: string;
}): Promise<Result<ITask[], "BAD_AUTH" | "BAD_CONNECTION">> {
  const response = await fetchWrapper("/api/tasks", {
    method: "GET",
    headers: { token: args.token },
  });
  if (response.k === "err") return response;

  if (!response.value.ok) {
    return {
      k: "err",
      value: "BAD_AUTH",
    };
  }

  return { k: "ok", value: await response.value.json() };
}

export async function updateTask(args: {
  token: string;
  taskName: string;
  doneBy: string;
}): Promise<Result<ITask, "BAD_AUTH" | "BAD_CONNECTION">> {
  const response = await fetchWrapper(
    `/api/tasks/actions/mark_task_done/${encodeURIComponent(
      args.taskName
    )}?by=${encodeURIComponent(args.doneBy)}`,
    { method: "POST", headers: { token: args.token } }
  );
  if (response.k === "err") return response;

  if (!response.value.ok) {
    return {
      k: "err",
      value: "BAD_AUTH",
    };
  }

  return { k: "ok", value: await response.value.json() };
}
