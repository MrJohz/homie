import {
  differenceInCalendarDays,
  isToday,
  isTomorrow,
  isYesterday,
} from "date-fns";

const dateFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: "short",
});
const dateSpanFormatter = new Intl.RelativeTimeFormat(undefined, {
  style: "narrow",
  numeric: "auto",
});

export function formatDate(date: Date): string {
  if (isToday(date)) {
    return dateSpanFormatter.format(0, "day");
  } else if (isTomorrow(date)) {
    return dateSpanFormatter.format(1, "day");
  } else if (isYesterday(date)) {
    return dateSpanFormatter.format(-1, "day");
  } else {
    return dateFormatter.format(date);
  }
}

export function formatRelativeDate(date: Date): string {
  return dateSpanFormatter.format(
    differenceInCalendarDays(date, new Date()),
    "day"
  );
}
