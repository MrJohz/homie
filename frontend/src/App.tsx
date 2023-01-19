import { BsChevronDown } from "solid-icons/bs";
import styles from "./App.module.css";
import { Header } from "./design/Header";
import { ITask } from "./types";
import { TaskList } from "./components/TaskList";
import { FlexGap } from "./design/FlexGap";
import { IconButton } from "./design/IconButton";
import { Modal, ModalActions, ModalHeader } from "./design/Modal";
import { Button } from "./design/Button";
import { createSignal } from "solid-js";

export function App() {
  const [open, setOpen] = createSignal(true);

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

  return (
    <div class={styles.page}>
      <Modal open={open()}>
        <ModalHeader>Login</ModalHeader>
        Hello, World!
        <ModalActions>
          <FlexGap />
          <Button variant="subtle" onClick={[setOpen, false]}>
            Cancel
          </Button>
          <Button>Login</Button>
        </ModalActions>
      </Modal>
      <Header>
        Tasks
        <FlexGap />
        <IconButton
          onClick={[setOpen, true]}
          aria-label="Open Menu"
          icon={<BsChevronDown aria-hidden="true" />}
        />
      </Header>
      <TaskList tasks={tasks} />
    </div>
  );
}
