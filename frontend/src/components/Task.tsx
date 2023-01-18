import { ITask } from "../types";
import { Button } from "../design/Button";
import styles from "./Task.module.css";
import { FlexGap } from "../design/FlexGap";

export function Task(props: { task: ITask }) {
  return (
    <div class={styles.taskbox}>
      <div class={styles.taskboxHeader}>
        {props.task.name} ({props.task.kind})
      </div>
      <div class={styles.taskboxItem}>
        Currently assigned to {props.task.assigned_to}
      </div>
      <div class={styles.taskboxItem}>
        Deadline...
        <FlexGap />
        <Button>Done</Button>
      </div>
    </div>
  );
}
