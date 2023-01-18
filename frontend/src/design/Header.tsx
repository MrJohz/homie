import { JSX } from "solid-js";
import styles from "./Header.module.css";

export function Header(props: { children?: JSX.Element }) {
  return <div class={styles.header}>{props.children}</div>;
}
