// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

import { ITask, TaskId } from "./types";

export type Success<T> = { k: "ok"; value: T };
export type Error<T> = { k: "err"; value: T };
export type Result<T, U> = Success<T> | Error<U>;

type ExtendedRequestInit = Omit<RequestInit, "body"> & {
  body?: any;
};

async function fetchWrapper(
  url: string,
  details: ExtendedRequestInit
): Promise<Result<Response, ["BAD_CONNECTION", string]>> {
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
    return { k: "err", value: ["BAD_CONNECTION", "could not connect to API"] };
  }
}

export async function createToken(
  username: string,
  password: string
): Promise<
  | Success<string>
  | Error<
      ["BAD_CONNECTION", string] | ["BAD_AUTH", string] | ["BAD_SERVER", string]
    >
> {
  const response = await fetchWrapper("/api/auth/login", {
    method: "POST",
    body: { username, password },
  });
  if (response.k === "err") return response;

  if (!response.value.ok) {
    const error = await response.value.text();
    return {
      k: "err",
      value: error.includes("token")
        ? ["BAD_AUTH", error]
        : ["BAD_SERVER", error],
    };
  }

  return { k: "ok", value: await response.value.json() };
}

export async function fetchTasks(args: {
  token: string;
}): Promise<
  Result<
    ITask[],
    ["BAD_CONNECTION", string] | ["BAD_AUTH", string] | ["BAD_SERVER", string]
  >
> {
  const response = await fetchWrapper("/api/tasks", {
    method: "GET",
    headers: { token: args.token },
  });
  if (response.k === "err") return response;

  if (!response.value.ok) {
    const error = await response.value.text();
    return {
      k: "err",
      value: error.includes("token")
        ? ["BAD_AUTH", error]
        : ["BAD_SERVER", error],
    };
  }

  return { k: "ok", value: await response.value.json() };
}

export async function updateTask(args: {
  token: string;
  taskId: TaskId;
  doneBy: string;
}): Promise<
  Result<
    ITask,
    ["BAD_CONNECTION", string] | ["BAD_AUTH", string] | ["BAD_SERVER", string]
  >
> {
  const response = await fetchWrapper(
    `/api/tasks/actions/mark_task_done/${args.taskId}?by=${encodeURIComponent(
      args.doneBy
    )}`,
    { method: "POST", headers: { token: args.token } }
  );
  if (response.k === "err") return response;

  if (!response.value.ok) {
    const error = await response.value.text();
    return {
      k: "err",
      value: error.includes("token")
        ? ["BAD_AUTH", error]
        : ["BAD_SERVER", error],
    };
  }

  return { k: "ok", value: await response.value.json() };
}
