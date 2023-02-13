// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

/* @refresh reload */
import { render } from "solid-js/web";
import { App } from "./App";
import "./index.css";
import { AuthProvider } from "./stores/useAuth";

render(
  () => (
    <AuthProvider>
      <App />
    </AuthProvider>
  ),
  document.getElementById("root") as HTMLElement
);
