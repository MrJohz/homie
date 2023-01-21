import { createSignal, Show } from "solid-js";
import { Button } from "../design/Button";
import { Error } from "../design/Error";
import { Form, validateNonEmptyString } from "../design/Form";
import { InputRow } from "../design/InputRow";
import { Modal, ModalHeader, ModalActions } from "../design/Modal";
import { useAuth } from "../stores/useAuth";

import styles from "./LoginModal.module.css";

export function LoginModal() {
  const [auth, authActions] = useAuth();
  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [error, setError] = createSignal<string | null>(null);

  async function login() {
    const response = await authActions.login(username(), password());

    if (response.k === "ok") {
      setError(null);
      setUsername("");
      setPassword("");
      return;
    }

    if (response.value === "BAD_AUTH") {
      setError("Invalid login details");
    } else if (response.value === "BAD_CONNECTION") {
      setError("Server error, try again");
    }
  }

  return (
    <Show when={auth().state === "unauthed"}>
      <Modal open>
        <ModalHeader>Login</ModalHeader>
        <Form onSubmit={login}>
          <InputRow
            label="Username"
            value={username()}
            onInput={(e) => setUsername(e.currentTarget.value)}
            validate={[validateNonEmptyString]}
          />
          <InputRow
            label="Password"
            value={password()}
            type="password"
            onInput={(e) => setPassword(e.currentTarget.value)}
            validate={[validateNonEmptyString]}
          />
          <ModalActions class={styles.actions}>
            <Error class={styles.error} error={error()} />
            <Button type="submit" variant="subtle">
              Login
            </Button>
          </ModalActions>
        </Form>
      </Modal>
    </Show>
  );
}
