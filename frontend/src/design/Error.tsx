import clsx from "clsx";
import { BsExclamationCircle } from "solid-icons/bs";
import { Show } from "solid-js";

import styles from "./Error.module.css";

export function Error(props: { class?: string; error?: string | null }) {
  return (
    <Show when={props.error !== null}>
      <span class={clsx(styles.error, props.class)}>
        <BsExclamationCircle />
        {props.error}
      </span>
    </Show>
  );
}
