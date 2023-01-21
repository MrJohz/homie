import styles from "./App.module.css";
import { Header } from "./design/Header";
import { TaskList } from "./components/TaskList";
import { useAuth } from "./stores/useAuth";
import { fetchTasks } from "./resources";
import { createStore, reconcile } from "solid-js/store";
import { ITask } from "./types";
import { createEffect, onCleanup, Show } from "solid-js";
import { LoginModal } from "./components/LoginModal";

export function App() {
  const [auth, authActions] = useAuth();
  const [tasks, setTasks] = createStore<ITask[]>([]);

  function refreshTasks() {
    if (auth().state === "unauthed") {
      setTasks([]);
      return;
    }

    authActions.fetchWithToken(fetchTasks, {}).then((tasks) => {
      if (tasks.k !== "ok") return console.error(tasks.value);

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
      <TaskList tasks={tasks} />
    </div>
  );
}
