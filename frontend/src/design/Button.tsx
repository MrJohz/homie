import clsx from "clsx";
import { splitProps } from "solid-js";
import { JSX } from "solid-js/jsx-runtime";
import styles from "./Button.module.css";

export function Button(
  props: JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
    variant?: "subtle" | "default";
    loading?: boolean;
  }
) {
  const [own, rest] = splitProps(props, ["variant", "class", "loading"]);
  return (
    <button
      {...rest}
      class={clsx(
        {
          [styles.subtle]: props.variant === "subtle",
          [styles.default]:
            props.variant === "default" || props.variant == null,
        },
        own.class
      )}
    />
  );
}
