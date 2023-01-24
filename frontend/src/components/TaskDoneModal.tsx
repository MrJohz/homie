import { createEffect, createSignal, on } from "solid-js";
import { Button } from "../design/Button";
import { Error } from "../design/Error";
import { Form } from "../design/Form";
import { InputRow } from "../design/InputRow";
import { Modal, ModalActions, ModalHeader } from "../design/Modal";
import { updateTask } from "../resources";
import { useAuth } from "../stores/useAuth";
import { ITask } from "../types";

export function TaskDoneModal(props: {
  task: ITask;
  open: boolean;
  onCancel: () => void;
  onConfirm: (doneBy: ITask) => void;
}) {
  const [, authActions] = useAuth();
  const [doneBy, setDoneBy] = createSignal(props.task.assigned_to);
  const [error, setError] = createSignal<string | null>(null);

  createEffect(
    on(
      () => [props.open, props.task.assigned_to],
      () => {
        setDoneBy(props.task.assigned_to);
      }
    )
  );

  return (
    <Modal open={props.open} onCancel={props.onCancel}>
      <ModalHeader>Done</ModalHeader>
      <Form
        onSubmit={async () => {
          const result = await authActions.fetchWithToken(updateTask, {
            taskName: props.task.name,
            doneBy: doneBy(),
          });
          if (result.k == "err") {
            return setError(
              result.value[0] === "BAD_AUTH"
                ? "Unauthorized"
                : "Connection down"
            );
          }

          props.onConfirm(result.value);
        }}
      >
        <InputRow
          type="select"
          label="Done by"
          value={doneBy()}
          items={props.task.participants}
          onChange={(e) => setDoneBy(e.currentTarget.value)}
        />
        <ModalActions>
          <Button type="reset" variant="subtle" onClick={props.onCancel}>
            Cancel
          </Button>
          <Error mergeRight error={error()} />
          <Button type="submit">Done</Button>
        </ModalActions>
      </Form>
    </Modal>
  );
}
