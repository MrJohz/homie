import { BsChevronDown } from "solid-icons/bs";
import styles from "./App.module.css";
import { Header } from "./design/Header";
import { ITask } from "./types";
import { TaskList } from "./components/TaskList";
import { FlexGap } from "./design/FlexGap";
import { IconButton } from "./design/IconButton";
import { Modal, ModalActions, ModalHeader } from "./design/Modal";
import { Button } from "./design/Button";
import { useAuth } from "./stores/useAuth";
import { fetchTasks } from "./resources";
import { createEffect } from "solid-js";

export function App() {
  const [auth, authActions] = useAuth();

  const tasks: ITask[] = [
    {
      name: "Clean the kitchen",
      kind: "Schedule",
      assigned_to: "Jonathan",
      deadline: { Upcoming: 3 },
    },
    {
      name: "Clean the kitchen",
      kind: "Schedule",
      assigned_to: "Jonathan",
      deadline: { Upcoming: 3 },
    },
    {
      name: "Clean the kitchen",
      kind: "Schedule",
      assigned_to: "Jonathan",
      deadline: { Upcoming: 3 },
    },
    {
      name: "Clean the kitchen",
      kind: "Schedule",
      assigned_to: "Jonathan",
      deadline: { Upcoming: 3 },
    },
    {
      name: "Clean the kitchen",
      kind: "Schedule",
      assigned_to: "Jonathan",
      deadline: { Upcoming: 3 },
    },
    {
      name: "Clean the kitchen",
      kind: "Schedule",
      assigned_to: "Jonathan",
      deadline: { Upcoming: 3 },
    },
  ];

  const result = authActions.fetchWithToken(fetchTasks, {});

  return (
    <div class={styles.page}>
      <Modal open={auth().state === "unauthed"}>
        <ModalHeader>Login</ModalHeader>
        Hello, World!
        <ModalActions>
          <FlexGap />
          <Button onClick={() => authActions.login("Test User", "testpw123")}>
            Login
          </Button>
        </ModalActions>
      </Modal>
      <Header>
        Tasks
        <FlexGap />
        <IconButton
          aria-label="Open Menu"
          icon={<BsChevronDown aria-hidden="true" />}
        />
      </Header>
      <TaskList tasks={tasks} />
    </div>
  );
}
