import { createSignal, For } from "solid-js";
import { ITask } from "../types";
import {
  DEFAULT_SORT_FILTER,
  SortAndFilter,
  SortFilter,
} from "./SortAndFilter";
import { Task } from "./Task";
import styles from "./TaskList.module.css";

export function TaskList(props: {
  tasks: ITask[];
  onUpdate: (tasks: ITask[]) => void;
}) {
  const [filter, setFilter] = createSignal<SortFilter>(DEFAULT_SORT_FILTER);
  return (
    <div class={styles.taskList}>
      <SortAndFilter tasks={props.tasks} onChange={setFilter} />
      <For each={filter()(props.tasks)}>
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
