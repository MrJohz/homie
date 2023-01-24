import { createEffect, createSignal, onCleanup, Show } from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import styles from "./App.module.css";
import { LoginModal } from "./components/LoginModal";
import { TaskList } from "./components/TaskList";
import { Header } from "./design/Header";
import { fetchTasks } from "./resources";
import { useAuth } from "./stores/useAuth";
import { ITask } from "./types";

export function App() {
  const [auth, authActions] = useAuth();
  const [tasks, setTasks] = createStore<ITask[]>([]);
  const [error, setError] = createSignal<{ code: string; text: string } | null>(
    null
  );

  function refreshTasks() {
    if (auth().state === "unauthed") {
      setTasks([]);
      return;
    }

    authActions.fetchWithToken(fetchTasks, {}).then((tasks) => {
      if (tasks.k !== "ok") {
        const [code, text] = tasks.value;
        setError({ code, text });
        return;
      }

      setError(null);
      setTasks(reconcile(tasks.value, { key: "name", merge: true }));
    });
  }

  createEffect(() => {
    refreshTasks();
    const interval = setInterval(refreshTasks, 10 * 60 * 1000);
    onCleanup(() => clearInterval(interval));
  });

  return (
    <div class={styles.page}>
      <LoginModal />
      <Header
        menu={[
          {
            type: "link",
            name: "Jellyfin",
            url: "http://192.168.0.138:8096/web/index.html",
          },
          {
            type: "action",
            name: "Logout",
            onClick: () => authActions.logout(),
          },
        ]}
      >
        Tasks
      </Header>
      <Show when={error() !== null}>
        <div>
          {error()?.code} -- {error()?.text}
        </div>
      </Show>
      <TaskList
        tasks={tasks}
        onUpdate={(newTasks) =>
          setTasks(reconcile(newTasks, { key: "name", merge: true }))
        }
      />
    </div>
  );
}
