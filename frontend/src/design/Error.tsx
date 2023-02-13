// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

import clsx from "clsx";
import { BsExclamationCircle } from "solid-icons/bs";
import { Show } from "solid-js";

import styles from "./Error.module.css";

export function Error(props: {
  class?: string;
  error?: string | null;
  mergeRight?: boolean;
}) {
  return (
    <Show when={props.error !== null}>
      <span
        class={clsx(
          styles.error,
          props.class,
          props.mergeRight && styles.mergeRight
        )}
      >
        <BsExclamationCircle />
        {props.error}
      </span>
    </Show>
  );
}
