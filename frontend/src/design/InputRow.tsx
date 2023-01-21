import clsx from "clsx";
import {
  BsExclamation,
  BsExclamationCircle,
  BsExclamationLg,
} from "solid-icons/bs";
import {
  createEffect,
  createSignal,
  JSX,
  onCleanup,
  Show,
  splitProps,
  useContext,
} from "solid-js";
import { FormContext, ValidationArray } from "./Form";

import styles from "./InputRow.module.css";

export function InputRow(
  props: JSX.InputHTMLAttributes<HTMLInputElement> & {
    label: JSX.Element;
    validate?: ValidationArray;
  }
) {
  const [own, rest] = splitProps(props, ["label", "class", "validate"]);
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
      <label class={styles.container}>
        <span class={styles.label}>{own.label}</span>
        <input {...rest} ref={setRef} class={clsx(own.class, styles.input)} />
        <Show when={error() !== null}>
          <span class={styles.error}>
            <BsExclamationCircle />
            {error()}
          </span>
        </Show>
      </label>
    </>
  );
}
