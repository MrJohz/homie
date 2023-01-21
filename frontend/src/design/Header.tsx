import { BsChevronDown, BsChevronUp } from "solid-icons/bs";
import { createSignal, For, JSX, Show } from "solid-js";
import styles from "./Header.module.css";
import { IconButton } from "./IconButton";

export type MenuItem =
  | { type: "link"; name: string; url: string }
  | {
      type: "action";
      name: string;
      onClick: (
        e: MouseEvent & { currentTarget: HTMLButtonElement; target: Element }
      ) => void;
    };

export function Header(props: { children?: JSX.Element; menu: MenuItem[] }) {
  const [open, setOpen] = createSignal(false);
  return (
    <div class={styles.header}>
      <div class={styles.main}>
        {props.children}
        <IconButton
          aria-label="Open Menu"
          onClick={() => setOpen((prev) => !prev)}
          icon={
            open() ? (
              <BsChevronUp aria-hidden="true" />
            ) : (
              <BsChevronDown aria-hidden="true" />
            )
          }
        />
      </div>
      <Show when={open()}>
        <For each={props.menu}>
          {(item) =>
            item.type === "link" ? (
              <a href={item.url}>{item.name}</a>
            ) : (
              <button
                onClick={(e) => {
                  item.onClick(e);
                  if (!e.defaultPrevented) {
                    setOpen(false);
                  }
                }}
              >
                {item.name}
              </button>
            )
          }
        </For>
      </Show>
    </div>
  );
}
