import { For } from "solid-js";
import { ITask } from "../types";
import { Task } from "./Task";
import styles from "./TaskList.module.css";

export function TaskList(props: { tasks: ITask[] }) {
  return (
    <div class={styles.taskList}>
      <For each={props.tasks}>{(task) => <Task task={task} />}</For>
    </div>
  );
}
