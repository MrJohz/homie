/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { App } from "./App";
import { AuthProvider } from "./stores/useAuth";

render(
  () => (
    <AuthProvider>
      <App />
    </AuthProvider>
  ),
  document.getElementById("root") as HTMLElement
);
