// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

import { createContext, JSX } from "solid-js";
import { createStore, produce } from "solid-js/store";
import styles from "./Form.module.css";

export const FormContext = createContext<FormHandle | null>(null);
export type ValidationArray = Array<(input: string) => false | string>;
export type SetError = (error: string | null) => void;
export type FormHandle = {
  addInput(
    ref: HTMLInputElement,
    validations: ValidationArray,
    errorHandle: SetError
  ): () => void;
};

type State = {
  validationCallbacks: Array<() => string | null>;
};

function runValidations(
  input: string,
  validations: ValidationArray
): string | null {
  for (const validation of validations) {
    const hasError = validation(input);
    if (hasError === false) continue;
    return hasError;
  }
  return null;
}

export function Form(props: {
  children: JSX.Element;
  onSubmit?: JSX.HTMLAttributes<HTMLFormElement>["onSubmit"];
}) {
  const [state, setState] = createStore<State>({ validationCallbacks: [] });

  return (
    <form
      class={styles.form}
      onSubmit={(e) => {
        e.preventDefault();
        let allowed = true;
        for (const callback of state.validationCallbacks) {
          const maybeError = callback();
          if (maybeError !== null) {
            allowed = false;
          }
        }
        if (!allowed) return;
        if (!props.onSubmit) return;

        if (typeof props.onSubmit === "function") {
          props.onSubmit(e as any);
        } else {
          props.onSubmit[0](props.onSubmit[1], e as any);
        }
      }}
    >
      <FormContext.Provider
        children={props.children}
        value={{
          addInput(input, validations, errorHandle) {
            let error: string | null = null;
            const validate = () => {
              error = runValidations(input.value, validations);
              errorHandle(error);
              return error;
            };
            const onChange = validate;
            const onInput = () => {
              // only validate on input if the user has already seen
              // an error in this field, otherwise wait until they have finished typing
              if (error === null) return;
              validate();
            };
            setState(
              produce((state) => {
                state.validationCallbacks.push(validate);
              })
            );
            input.addEventListener("change", onChange);
            input.addEventListener("blur", onChange);
            input.addEventListener("input", onInput);
            return () => {
              input.removeEventListener("change", onChange);
              input.removeEventListener("blur", onChange);
              input.removeEventListener("input", onInput);
              setState(
                produce((state) => {
                  const index = state.validationCallbacks.indexOf(validate);
                  if (index === -1) return;
                  state.validationCallbacks.splice(index, 1);
                })
              );
            };
          },
        }}
      />
    </form>
  );
}

export function validateNonEmptyString(input: string): false | string {
  if (!input.length) return "Field must not be empty";
  return false;
}
