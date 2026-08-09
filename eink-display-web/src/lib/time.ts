const TZ = "Australia/Perth";

const updatedFormatter = new Intl.DateTimeFormat("en-AU", {
  timeZone: TZ,
  weekday: "short",
  hour: "2-digit",
  minute: "2-digit",
  hour12: false,
});

export function formatUpdatedAt(date: Date) {
  return updatedFormatter.format(date);
}

const dayFormatter = new Intl.DateTimeFormat("en-CA", {
  timeZone: TZ,
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
});

function perthDay(date: Date) {
  return dayFormatter.format(date);
}

export function perthMidnightISO() {
  return new Date(`${perthDay(new Date())}T00:00:00+08:00`).toISOString();
}

export function fromToday<T extends { readonly dateTime: string }>(days: readonly T[]) {
  const today = perthDay(new Date());
  const start = days.findIndex((d) => perthDay(new Date(d.dateTime)) >= today);

  return start === -1 ? [] : days.slice(start);
}
