import { createSignal, Show } from "solid-js";
import { createStore } from "solid-js/store";
import { Button } from "../design/Button";
import { FlexGap } from "../design/FlexGap";
import { Form, validateNonEmptyString } from "../design/Form";
import { InputRow } from "../design/InputRow";
import { Modal, ModalHeader, ModalActions } from "../design/Modal";
import { useAuth } from "../stores/useAuth";

type Errors = {
  username: string | undefined;
  password: string | undefined;
  form: string | undefined;
};

export function LoginModal() {
  const [auth, authActions] = useAuth();
  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [error, setError] = createStore<Errors>({
    username: undefined,
    password: undefined,
    form: undefined,
  });

  async function login() {
    let missingData = false;
    if (/^\s*$/.test(username())) {
      setError({ username: "Username must not be empty" });
      missingData = true;
    } else {
      setError({ username: undefined });
    }
    if (/^\s*$/.test(password())) {
      setError({ password: "Password must not be empty" });
      missingData = true;
    } else {
      setError({ password: undefined });
    }

    if (missingData) return;
    const response = await authActions.login(username(), password());
    setError({ username: undefined, password: undefined, form: undefined });

    if (response.k == "ok") {
      setUsername("");
      setPassword("");
      return;
    }

    if (response.value === "BAD_AUTH") {
      setError({ form: "Invalid username and/or password" });
    } else if (response.value === "BAD_CONNECTION") {
      setError({ form: "Could not connect to servers, please try again" });
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
          <ModalActions>
            <FlexGap />
            <Button type="submit" variant="subtle">
              Login
            </Button>
          </ModalActions>
        </Form>
      </Modal>
    </Show>
  );
}
