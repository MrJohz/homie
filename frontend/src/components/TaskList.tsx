import { For } from "solid-js";
import { ITask } from "../types";
import { Task } from "./Task";
import styles from "./TaskList.module.css";

export function TaskList(props: {
  tasks: ITask[];
  onUpdate: (tasks: ITask[]) => void;
}) {
  return (
    <div class={styles.taskList}>
      <For each={props.tasks}>
        {(task) => (
          <Task
            task={task}
            onUpdate={(task) => {
              props.onUpdate(
                props.tasks.map((oldTask) =>
                  oldTask.name === task.name ? task : oldTask
                )
              );
            }}
          />
        )}
      </For>
    </div>
  );
}
