import { JSX, splitProps } from "solid-js";
import styles from "./IconButton.module.css";

export function IconButton(
  props: JSX.HTMLAttributes<HTMLButtonElement> & { icon: JSX.Element }
) {
  const [own, rest] = splitProps(props, ["icon"]);
  return (
    <button {...rest} class={styles.iconButton}>
      {own.icon}
    </button>
  );
}
