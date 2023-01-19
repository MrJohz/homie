import { BsChevronDown } from "solid-icons/bs";
import styles from "./App.module.css";
import { Header } from "./design/Header";
import { ITask } from "./types";
import { TaskList } from "./components/TaskList";
import { FlexGap } from "./design/FlexGap";
import { IconButton } from "./design/IconButton";

export function App() {
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
  ];

  return (
    <div class={styles.page}>
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
