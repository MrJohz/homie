import clsx from "clsx";
import { createEffect, createSignal, JSX } from "solid-js";

import styles from "./Modal.module.css";

const SELF_CLOSE_SENTINEL = "SELF_CLOSE_SENTINEL";

function callHandler<T, E extends Event>(
  handler: JSX.EventHandlerUnion<T, E> | undefined,
  event: E & {
    currentTarget: T;
    target: Element;
  }
) {
  if (handler == null) return;

  if ("0" in handler) {
    return handler[0](handler[1], event);
  } else {
    return handler(event);
  }
}

export function Modal(props: {
  open?: boolean;
  onCancel?: JSX.DialogHtmlAttributes<HTMLDialogElement>["onCancel"];
  children?: JSX.Element;
}) {
  const [dialogRef, setDialogRef] = createSignal<HTMLDialogElement>();

  createEffect(() => {
    if (props.open) {
      dialogRef().showModal();
    } else {
      dialogRef().close(SELF_CLOSE_SENTINEL);
    }
  });

  return (
    <dialog
      class={styles.modalWrapper}
      ref={setDialogRef}
      onClick={(e) => {
        if (e.target !== e.currentTarget) return; // bubbling is happening

        // At this point, there is no pixel of "dialog-element" that isn't covered
        // by another child div, therefore the user must have clicked on the
        // backdrop (see e.g. https://stackoverflow.com/a/40551169)
        e.preventDefault();
        e.currentTarget.dispatchEvent(
          new Event("cancel", {
            bubbles: false,
            cancelable: true,
            composed: false,
          })
        );
      }}
      onClose={(e) => {
        if (e.currentTarget.returnValue === SELF_CLOSE_SENTINEL) return;

        e.preventDefault();
        e.currentTarget.showModal();
      }}
      onCancel={(e) => {
        e.preventDefault();
        callHandler(props.onCancel, e);
      }}
    >
      <div class={styles.modal}>{props.children}</div>
    </dialog>
  );
}

export function ModalHeader(props: JSX.HTMLAttributes<HTMLDivElement>) {
  return <div {...props} class={clsx(styles.modalHeader, props.class)} />;
}

export function ModalActions(props: JSX.HTMLAttributes<HTMLDivElement>) {
  return <div {...props} class={clsx(styles.modalActions, props.class)} />;
}
