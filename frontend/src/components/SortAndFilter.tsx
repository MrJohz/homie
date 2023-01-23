import { add, isBefore } from "date-fns";
import { createSignal } from "solid-js";
import { createStore } from "solid-js/store";
import { ITask } from "../types";

export type SortFilter = (tasks: ITask[]) => ITask[];

type Direction = "ASC" | "DESC";
type SortSettings = {
  field: "deadline" | "name";
  direction: Direction;
};
type FilterSettings = {
  assignee: false | string;
};

function timeLeft(task: ITask): number {
  if ("Overdue" in task.deadline) return -task.deadline.Overdue;
  return task.deadline.Upcoming;
}

function generateSortFilter(
  sort: SortSettings,
  filter: FilterSettings
): SortFilter {
  return (tasks) => {
    return tasks
      .filter((task) => {
        if (filter.assignee === false) return true;
        if (task.assigned_to !== filter.assignee) return false;

        return true;
      })
      .sort((task1, task2) => timeLeft(task1) - timeLeft(task2));
  };
}

export const DEFAULT_SORT_FILTER = generateSortFilter(
  { field: "deadline", direction: "DESC" },
  { assignee: false }
);

export function SortAndFilter(props: {
  tasks: ITask[];
  onChange: (sortFilter: SortFilter) => void;
}) {
  const [open, setOpen] = createSignal(false);
  const [sortSettings, setSortSettings] = createStore<SortSettings>({
    field: "deadline",
    direction: "DESC",
  });
  const [filterSettings, setFilterSettings] = createStore<FilterSettings>({
    assignee: false,
  });

  const summaryString = () => {
    const parts = ["Showing "];
    if (filterSettings.assignee === false) {
      parts.push("all tasks");
    } else {
      parts.push("tasks assigned to", filterSettings.assignee);
    }
    return parts;
  };
  return <div>{summaryString()}</div>;
}
