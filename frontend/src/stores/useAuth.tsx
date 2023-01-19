import {
  Accessor,
  createContext,
  createEffect,
  createSignal,
  JSX,
  useContext,
} from "solid-js";
import { createToken, Result } from "../resources";

const STORAGE_KEY = "homie::token";

type AuthState =
  | { state: "authed"; username: string; token: string }
  | { state: "unauthed" };

function stateFromLocalStorage(): AuthState {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored == null) {
    return { state: "unauthed" };
  } else {
    try {
      const [username, token] = JSON.parse(stored);
      return { state: "authed", username, token };
    } catch (e) {
      return { state: "unauthed" };
    }
  }
}

function stateToLocalStorage(state: AuthState) {
  if (state.state === "unauthed") {
    localStorage.removeItem(STORAGE_KEY);
  } else {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify([state.username, state.token])
    );
  }
}

type AuthActions = {
  login(
    username: string,
    password: string
  ): Promise<Result<void, "BAD_CONNECTION" | "BAD_AUTH">>;
  fetchWithToken<Arg extends Record<string, unknown>, T, Err extends string>(
    func: (arg: Arg & { token: string }) => Promise<Result<T, Err>>,
    args: Arg
  ): Promise<Result<T, Err | "BAD_AUTH">>;
};

const AuthContext = createContext<readonly [Accessor<AuthState>, AuthActions]>([
  () => ({ state: "unauthed" as const }),
  {
    login: () => {
      throw new Error("not usable");
    },
    fetchWithToken: () => {
      throw new Error("not usable");
    },
  } satisfies AuthActions,
]);

export function AuthProvider(props: { children?: JSX.Element }) {
  const [state, setState] = createSignal<AuthState>(stateFromLocalStorage());

  createEffect(() => {
    stateToLocalStorage(state());
  });

  return (
    <AuthContext.Provider
      children={props.children}
      value={[
        state,
        {
          async login(
            username: string,
            password: string
          ): Promise<Result<void, "BAD_CONNECTION" | "BAD_AUTH">> {
            const status = await createToken(username, password);
            if (status.k !== "ok") return status;

            setState({ state: "authed", username, token: status.value });
            return { k: "ok", value: undefined };
          },
          async fetchWithToken(func, args) {
            const authState = state();
            if (authState.state === "unauthed")
              return { k: "err", value: "BAD_AUTH" };

            const status = await func({ ...args, token: authState.token });
            if (status.k === "ok" || status.value !== "BAD_AUTH") return status;

            setState({ state: "unauthed" });
            return status;
          },
        },
      ]}
    />
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
