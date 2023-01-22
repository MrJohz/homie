import clsx from "clsx";
import {
  BsExclamation,
  BsExclamationCircle,
  BsExclamationLg,
} from "solid-icons/bs";
import {
  createEffect,
  createSignal,
  For,
  JSX,
  Match,
  onCleanup,
  Show,
  splitProps,
  Switch,
  useContext,
} from "solid-js";
import { Error } from "./Error";
import { FormContext, ValidationArray } from "./Form";

import styles from "./InputRow.module.css";

type InputRowProps =
  | ({
      type: "text" | "email" | "password";
    } & JSX.InputHTMLAttributes<HTMLInputElement>)
  | ({
      type: "select";
    } & JSX.SelectHTMLAttributes<HTMLSelectElement>);

export function InputRow(
  props: InputRowProps & {
    label: JSX.Element;
    validate?: ValidationArray;
    items?: string[];
  }
) {
  const [own, rest] = splitProps(props, [
    "label",
    "class",
    "validate",
    "items",
    "type",
  ]);
  const [error, setError] = createSignal<string | null>(null);
  const [ref, setRef] = createSignal<HTMLInputElement>();
  const form = useContext(FormContext);

  createEffect(() => {
    if (!form) return;
    if (!own.validate) return;

    const r = ref();
    if (!r) return;

    const cleanup = form.addInput(r, own.validate, setError);
    onCleanup(cleanup);
  });

  return (
    <>
      <label class={clsx(styles.container, own.class)}>
        <span class={styles.label}>{own.label}</span>
        <Switch
          fallback={
            <input
              {...(rest as JSX.InputHTMLAttributes<HTMLInputElement>)}
              ref={setRef}
              class={styles.input}
              type={own.type}
            />
          }
        >
          <Match when={own.type === "select"}>
            <select
              ref={setRef}
              class={styles.input}
              {...(rest as JSX.SelectHTMLAttributes<HTMLSelectElement>)}
            >
              <For each={own.items}>
                {(item) => (
                  <option value={item} selected={rest.value === item}>
                    {item}
                  </option>
                )}
              </For>
            </select>
          </Match>
        </Switch>
        <Error error={error()} />
      </label>
    </>
  );
}
