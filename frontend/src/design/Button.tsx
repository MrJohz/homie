import { JSX } from "solid-js/jsx-runtime";
import styles from "./Button.module.css";

export function Button(props: JSX.HTMLAttributes<HTMLButtonElement>) {
  return <button {...props} class={styles.button} />;
}
