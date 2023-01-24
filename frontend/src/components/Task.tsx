import clsx from "clsx";
import { add } from "date-fns";
import { createSignal } from "solid-js";
import { Button } from "../design/Button";
import { FlexGap } from "../design/FlexGap";
import { formatDate, formatRelativeDate } from "../stores/formatting";
import { ITask } from "../types";
import styles from "./Task.module.css";
import { TaskDoneModal } from "./TaskDoneModal";

export function Task(props: { task: ITask; onUpdate: (task: ITask) => void }) {
  const [element, setElement] = createSignal<HTMLDivElement | null>(null);
  const [isOpen, setOpen] = createSignal(false);
  const [isTransitioning, setTransitioning] = createSignal(false);
  const dueDate = () => {
    const lastCompleted = new Date(props.task.last_completed);
    if (props.task.kind === "Interval") {
      return (
        "due " +
        formatRelativeDate(add(lastCompleted, { days: props.task.length_days }))
      );
    } else {
      const start = add(lastCompleted, { days: 1 });
      const end = add(lastCompleted, { days: props.task.length_days });
      return `${formatDate(start)} – ${formatDate(end)}`;
    }
  };
  return (
    <div
      ref={setElement}
      class={styles.taskbox}
      style={{
        "margin-bottom": isTransitioning() ? `-41px` : undefined,
        opacity: isTransitioning() ? 0 : undefined,
      }}
    >
      <div class={styles.taskboxHeader}>
        <span>{props.task.name}</span>
        <span>{props.task.assigned_to}</span>
      </div>
      <div class={styles.taskboxItem}>
        <span
          class={clsx({ [styles.overdue]: "Overdue" in props.task.deadline })}
          textContent={dueDate()}
        />
        <FlexGap />
        <Button onClick={[setOpen, true]}>Done</Button>
      </div>
      <TaskDoneModal
        open={isOpen()}
        task={props.task}
        onCancel={() => setOpen(false)}
        onConfirm={(task) => {
          setOpen(false);
          const e = element();
          if (e == null) return props.onUpdate(task);

          e.style.marginBottom = -e.offsetHeight + "px";
          e.style.opacity = "0";
          e.addEventListener(
            "transitionend",
            () => {
              e.style.marginBottom = "";
              e.style.opacity = "";
              props.onUpdate(task);
            },
            {
              once: true,
            }
          );

          // props.onUpdate(task);
        }}
      />
    </div>
  );
}
